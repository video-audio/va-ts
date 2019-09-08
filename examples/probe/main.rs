extern crate va_ts as ts;

mod error;

use std::collections::HashSet;
use std::collections::VecDeque;
use std::net::{Ipv4Addr, UdpSocket};
use std::process;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use clap::{App, Arg};
use url::{Host, Url};

use error::{Error, Kind as ErrorKind, Result};

// struct Packet {
//     offset: usize,

//     /// presentation time stamp
//     pts: Option<Duration>,

//     /// decode time stamp
//     dts: Option<Duration>,

//     /// reusable buffer to collect payload
//     buf: Cursor<Vec<u8>>,
// }

// impl Packet {
//     fn new() -> Packet {
//         Packet {
//             offset: 0,
//             pts: None,
//             dts: None,
//             buf: Cursor::new(Vec::with_capacity(2048)),
//         }
//     }

//     #[inline(always)]
//     fn buf_reset(&mut self) {
//         self.buf.set_position(0);
//         self.buf.get_mut().clear();
//     }

//     #[inline(always)]
//     fn is_empty(&self) -> bool {
//         self.buf.position() == 0
//     }
// }

// struct Track {
//     /// ID/PID
//     ///   - PID for mpegts
//     ///   - ID for RTMP/HLS, DASH, MP4
//     id: u16,

//     // TODO: add codec
//     // codec: Codec,
//     pkt: Packet,
//     // dbg_file: File,
// }

// impl Track {
//     fn new(id: u16) -> Track {
//         Track {
//             id: id,
//             pkt: Packet::new(),
//             // dbg_file: File::create(format!("/tmp/dump-{}.h264", id)).unwrap(),
//         }
//     }
// }

// struct Stream {
//     /// current global offset
//     /// aka bytes-processed / bytes-readen
//     offset: usize,

//     tracks: Vec<Track>,
// }

// impl Stream {
//     fn new() -> Stream {
//         Stream {
//             offset: 0,
//             tracks: Vec::new(),
//         }
//     }
// }

// pub struct TS {
//     pat_buf: Cursor<Vec<u8>>,
//     pmt_buf: Cursor<Vec<u8>>,
//     sdt_buf: Cursor<Vec<u8>>,
//     eit_buf: Cursor<Vec<u8>>,

//     stream: Option<Stream>,
// }

// impl TS {
//     fn new() -> TS {
//         TS {
//             pat_buf: Cursor::new(Vec::with_capacity(ts::Packet::SZ)),
//             pmt_buf: Cursor::new(Vec::with_capacity(ts::Packet::SZ)),
//             sdt_buf: Cursor::new(Vec::with_capacity(ts::Packet::SZ)),
//             eit_buf: Cursor::new(Vec::with_capacity(384)),

//             stream: None,
//         }
//     }

//     fn pat(&self) -> Option<ts::PAT> {
//         if self.pat_buf.position() == 0 {
//             None
//         } else {
//             Some(ts::PAT::new(self.pat_buf.get_ref().as_slice()))
//         }
//     }

//     fn pmt(&self) -> Option<ts::PMT> {
//         if self.pmt_buf.position() == 0 {
//             None
//         } else {
//             Some(ts::PMT::new(self.pmt_buf.get_ref().as_slice()))
//         }
//     }

//     fn pmt_pid(&self) -> Option<u16> {
//         self.pat().and_then(|p| p.first_program_map_pid())
//     }

//     fn sdt(&self) -> Option<ts::SDT> {
//         if self.sdt_buf.position() == 0 {
//             None
//         } else {
//             Some(ts::SDT::new(self.sdt_buf.get_ref().as_slice()))
//         }
//     }

//     /// are PAT, PMT, \[SDT\], stream builded?
//     fn can_demux(&self) -> bool {
//         self.pat().is_some() && self.pmt().is_some() && self.stream.is_some()
//     }
// }

// // struct DemuxerTS {}

// // impl DemuxerTS {
// //     pub fn new() -> DemuxerTS {
// //         DemuxerTS {}
// //     }
// // }

trait Input {
    fn open(&mut self) -> Result<()>;
    fn read(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
}

// trait Filter {
//     fn consume_strm(&self);
//     fn consume_trk(&self);
//     fn consume_pkt_raw(&self);
//     fn consume_pkt(&self);
//     fn consume_frm(&self);

//     fn produce_strm(&self);
//     fn produce_trk(&self);
//     fn produce_pkt_raw(&self);
//     fn produce_pkt(&self);
//     fn produce_frm(&self);
// }

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

impl ts::DemuxerEvents for DemuxerTSEvents {
    fn on_table(&mut self, id: ts::SubtableID, tbl: &ts::DemuxedTable) {
        if self.done_once.contains(&id) {
            return;
        } else {
            self.done_once.insert(id);
        }

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
                ts::SubtableID::EIT(..) => {
                    println!("{:?}", ts::EIT::new(raw));
                }
                ts::SubtableID::PMT(..) => {
                    println!("{:?}", ts::PMT::new(raw));
                }
            };
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

struct InputFile {
    url: Url,
}

impl InputFile {
    pub fn new(url: Url) -> InputFile {
        InputFile { url: url }
    }
}

impl Input for InputFile {
    fn open(&mut self) -> Result<()> {
        println!("<<< File open {}", self.url);

        Ok(())
    }

    fn read(&mut self) -> Result<()> {
        thread::sleep(Duration::from_secs(1000));

        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        println!("<<< File close {}", self.url);

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
        .version("0.0.1")
        .author("Ivan Egorov <vany.egorov@gmail.com>")
        .about("Video/audio swiss knife")
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

    let input_url_1 = input_url.clone();
    let input_url_2 = input_url.clone();

    // <input builder based on URL>
    let input_udp = InputUDP::new(input_url_1, 5000 * 7);
    let input_file = InputFile::new(input_url_2);
    // </input builder based on URL>

    let wrkr1 = Wrkr::new(input_udp);
    let wrkr2 = Wrkr::new(input_file);

    if let Err(err) = wrkr1.run() {
        eprintln!("error start worker №1: {:?}\n", err);
        process::exit(1);
    }

    if let Err(err) = wrkr2.run() {
        eprintln!("error start worker №2: {:?}\n", err);
        process::exit(1);
    }

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
