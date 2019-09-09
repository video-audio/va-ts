extern crate va_ts as ts;

mod error;

use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt;
use std::net::{Ipv4Addr, UdpSocket};
use std::process;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use clap::{App, Arg};
use url::{Host, Url};

use error::{Error, Kind as ErrorKind, Result};

trait Input {
    fn open(&mut self) -> Result<()>;
    fn read(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
}

struct DemuxerTSEvents {
    done_once: HashSet<ts::SubtableID>,
}

impl Default for DemuxerTSEvents {
    fn default() -> Self {
        DemuxerTSEvents {
            done_once: Default::default(),
        }
    }
}

struct EITFmt<'t>(&'t ts::DemuxedTable);

impl<'t> fmt::Display for EITFmt<'t> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for section_ref in self.0.sections.0.iter() {
            let section = (*section_ref).borrow();
            let raw = section.buf.0.get_ref().as_slice();

            let eit = ts::EIT::new(raw);

            for event in eit.events().filter_map(ts::Result::ok) {
                write!(
                    f,
                    "  {} ~ {}\n",
                    event.start_time(),
                    ts::DurationFmt::from(event.duration()),
                )?;

                if let Some(descs) = event.descriptors() {
                    for desc in descs
                        .filter_map(ts::Result::ok)
                        .filter(|d| d.is_dvb_short_event())
                    {
                        match desc.tag() {
                            ts::Tag::DVB(ts::TagDVB::ShortEvent) => {
                                let desc = ts::DescDVB0x4D::new(desc.buf_data());

                                let mut dst_buf = [0u8; 256];
                                let mut dst_str = std::str::from_utf8_mut(&mut dst_buf).unwrap();

                                match ts::AnnexA2::decode(desc.event_name(), &mut dst_str) {
                                    Ok(..) => write!(f, r#"    "{}""#, dst_str),
                                    Err(err) => write!(f, "  (error: {:?})", err),
                                }?;

                                dst_buf = [0u8; 256];
                                dst_str = std::str::from_utf8_mut(&mut dst_buf).unwrap();

                                match ts::AnnexA2::decode(desc.text(), &mut dst_str) {
                                    Ok(..) => write!(f, r#" "{}""#, dst_str),
                                    Err(err) => write!(f, " (error: {})", err),
                                }?;

                                write!(f, "\n")?;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl ts::DemuxerEvents for DemuxerTSEvents {
    fn on_table(&mut self, id: ts::SubtableID, tbl: &ts::DemuxedTable) {
        if self.done_once.contains(&id) {
            return;
        } else {
            self.done_once.insert(id);
        }

        match id {
            ts::SubtableID::EIT(..) => {
                print!(":EIT\n{}", EITFmt(tbl));
            }
            _ => {
                for section_ref in tbl.sections.0.iter() {
                    let section = (*section_ref).borrow();
                    let raw = section.buf.0.get_ref().as_slice();

                    match id {
                        ts::SubtableID::PAT(..) => {
                            println!("{:?}", ts::PAT::new(raw));
                        }
                        ts::SubtableID::SDT(..) => {
                            println!("{:?}", ts::SDT::new(raw));
                        }
                        ts::SubtableID::PMT(..) => {
                            println!("{:?}", ts::PMT::new(raw));
                        }
                        ts::SubtableID::EIT(..) => {
                            println!("{:?}", ts::EIT::new(raw));
                        }
                    };
                }
            }
        }
    }

    fn on_packet(&mut self, pkt: &ts::DemuxedPacket) {
        println!(
            "(0x{:016X}) :pid {:?} :pts {:?} :dts {:?} :sz {}",
            pkt.offset,
            pkt.pid,
            pkt.pts.map(ts::DurationFmt::from),
            pkt.dts.map(ts::DurationFmt::from),
            pkt.buf.sz(),
        );
    }
}

struct InputUDP {
    url: Url,

    // circullar-buffer / fifo
    buf: Arc<(Mutex<VecDeque<[u8; ts::Packet::SZ]>>, Condvar)>,

    demuxer: ts::Demuxer<DemuxerTSEvents>,
}

impl InputUDP {
    pub fn new(url: Url, buf_cap: usize) -> InputUDP {
        InputUDP {
            url: url,
            buf: Arc::new((Mutex::new(VecDeque::with_capacity(buf_cap)), Condvar::new())),

            demuxer: ts::Demuxer::new(Default::default()),
        }
    }
}

impl Input for InputUDP {
    fn open(&mut self) -> Result<()> {
        let input_host = self
            .url
            .host()
            .ok_or(Error::new(ErrorKind::InputUrlMissingHost))?;

        let input_port = self.url.port().unwrap_or(5500);

        let input_host_domain = match input_host {
            Host::Domain(v) => Ok(v),
            _ => Err(Error::new(ErrorKind::InputUrlHostMustBeDomain)),
        }?;

        let iface = Ipv4Addr::new(0, 0, 0, 0);
        // let socket = try!(UdpSocket::bind((input_host_domain, input_port)));;

        // let iface = Ipv4Addr::new(127, 0, 0, 1);
        println!(
            "[<] {:?}: {:?} @ {:?}",
            input_host_domain, input_port, iface
        );

        let input_host_ip_v4: Ipv4Addr = input_host_domain.parse().unwrap();

        let socket = UdpSocket::bind((input_host_domain, input_port))?;

        if let Err(e) = socket.join_multicast_v4(&input_host_ip_v4, &iface) {
            eprintln!("error join-multiocast-v4: {}", e);
        }

        let pair = self.buf.clone();
        thread::spawn(move || {
            let mut ts_pkt_raw: [u8; ts::Packet::SZ] = [0; ts::Packet::SZ];

            loop {
                // MTU (maximum transmission unit) == 1500 for Ethertnet
                // 7*ts::Packet::SZ = 7*188 = 1316 < 1500 => OK
                let mut pkts_raw = [0; 7 * ts::Packet::SZ];
                let (_, _) = socket.recv_from(&mut pkts_raw).unwrap();

                let &(ref lock, ref cvar) = &*pair;
                let mut buf = match lock.lock() {
                    Err(e) => {
                        eprintln!("lock and get buffer failed: {}", e);
                        continue;
                    }
                    Ok(buf) => buf,
                };

                for pkt_index in 0..7 * ts::Packet::SZ / ts::Packet::SZ {
                    let ts_pkt_raw_src =
                        &pkts_raw[pkt_index * ts::Packet::SZ..(pkt_index + 1) * ts::Packet::SZ];

                    ts_pkt_raw.copy_from_slice(ts_pkt_raw_src);
                    buf.push_back(ts_pkt_raw);
                }

                cvar.notify_all();
            }
        });

        Ok(())
    }

    fn read(&mut self) -> Result<()> {
        let pair = self.buf.clone();
        let &(ref lock, ref cvar) = &*pair;
        let mut buf = lock.lock().ok().ok_or(Error::new_with_details(
            ErrorKind::SyncPoison,
            "udp read lock error",
        ))?;

        buf = cvar.wait(buf).ok().ok_or(Error::new_with_details(
            ErrorKind::SyncPoison,
            "udp read cwar wait error",
        ))?;

        while !buf.is_empty() {
            let ts_pkt_raw = buf.pop_front().unwrap();

            if let Err(e) = self.demuxer.demux(&ts_pkt_raw) {
                eprintln!("error demux ts-packet: ({:?})", e);
            }
        }

        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        println!("<<< UDP close");

        Ok(())
    }
}

struct Wrkr<I> {
    input: Arc<Mutex<I>>,
}

impl<I> Wrkr<I>
where
    I: Input + std::marker::Send + 'static,
{
    pub fn new(input: I) -> Wrkr<I> {
        Wrkr {
            input: Arc::new(Mutex::new(input)),
        }
    }

    pub fn run(&self) -> Result<()> {
        let input = self.input.clone();
        {
            input.lock().unwrap().open()?;
        }

        thread::spawn(move || loop {
            match input.lock().unwrap().read() {
                Err(err) => {
                    eprintln!("error read {}", err);
                    return;
                }
                Ok(_) => {}
            }
        });

        Ok(())
    }
}

fn main() {
    // let args: Vec<String> = env::args().collect();
    // println!("{:?}", args);
    let matches = App::new("V/A tool")
        .version("0.0.3")
        .author("Ivan Egorov <vany.egorov@gmail.com>")
        .about("simple mpeg-ts mcast probe")
        .arg(
            Arg::with_name("input")
                // .index(1)
                .short("i")
                .long("input")
                .help("Sets the input file to use")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let input_raw = matches.value_of("input").unwrap();
    let input_url = match Url::parse(input_raw) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("error parse input url: {:?}\n", err);
            process::exit(1);
        }
    };

    let input = InputUDP::new(input_url, 5000 * 7);

    let wrkr = Wrkr::new(input);

    if let Err(err) = wrkr.run() {
        eprintln!("error start worker: {:?}\n", err);
        process::exit(1);
    }

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
