use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Cursor, Write};
use std::rc::Rc;
use std::time::Duration;

use crate::duration_fmt::DurationFmt;
use crate::packet::Packet;
use crate::pes::PES;
use crate::pid::PID;
use crate::result::Result;
use crate::section::WithSyntaxSection;
use crate::subtable_id::{SubtableID, SubtableIDer};
use crate::{EIT, PAT, PMT, SDT};

struct Buf(Cursor<Vec<u8>>);

impl Buf {
    #[inline(always)]
    fn reset(&mut self) {
        self.0.set_position(0);
        self.0.get_mut().clear();
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.0.position() == 0
    }
}

impl Default for Buf {
    fn default() -> Self {
        Buf(Cursor::new(Vec::with_capacity(2048)))
    }
}

struct Section {
    number: u8,
    buf: Buf,
}

impl Section {
    fn new(number: u8) -> Section {
        Section {
            number,
            buf: Default::default(),
        }
    }

    #[inline(always)]
    fn into_ref(self) -> SectionRef {
        Rc::new(RefCell::new(Box::new(self)))
    }
}

type SectionRef = Rc<RefCell<Box<Section>>>;

struct Sections(Vec<SectionRef>);

impl Sections {
    #[inline(always)]
    #[allow(dead_code)]
    fn get_mut(&mut self, number: u8) -> Option<&mut SectionRef> {
        self.0.iter_mut().find(|s| s.borrow().number == number)
    }

    #[inline(always)]
    fn push(&mut self, s: SectionRef) {
        self.0.push(s);
        self.0
            .sort_unstable_by(|a, b| a.borrow().number.cmp(&b.borrow().number));
    }
}

impl Default for Sections {
    fn default() -> Self {
        Sections(Vec::with_capacity(1))
    }
}

struct Table {
    #[allow(dead_code)]
    id: SubtableID,
    sections: Sections,
}

impl Table {
    fn new(id: SubtableID) -> Table {
        Table {
            id,
            sections: Default::default(),
        }
    }
}

struct Tables {
    map: HashMap<SubtableID, Table>,
    /// current demuxing section
    current: Option<SectionRef>,
}

impl Tables {}

impl Default for Tables {
    fn default() -> Self {
        Tables {
            map: HashMap::new(),
            current: None,
        }
    }
}

struct Stream {
    pid: PID,

    offset: usize,

    /// presentation time stamp
    pts: Option<Duration>,

    /// decode time stamp
    dts: Option<Duration>,

    #[allow(dead_code)]
    buf: Buf,
}

impl Stream {
    fn new(pid: PID) -> Stream {
        Stream {
            pid,
            offset: 0,
            pts: None,
            dts: None,
            buf: Default::default(),
        }
    }
}

#[derive(Default)]
struct Streams {
    #[allow(dead_code)]
    map: HashMap<PID, Stream>,
}

impl Streams {}

#[derive(Debug)]
struct PMTPids(Vec<PID>);

impl PMTPids {
    #[inline(always)]
    fn has(&self, pid: PID) -> bool {
        self.0.iter().any(|p| (*p) == pid)
    }

    #[inline(always)]
    fn push_uniq(&mut self, pid: PID) {
        if !self.has(pid) {
            self.0.push(pid)
        }
    }
}

impl Default for PMTPids {
    fn default() -> Self {
        PMTPids(Vec::with_capacity(3))
    }
}

/// TODO: use tree, redix tree here
/// TODO: add benches
pub struct Demuxer {
    offset: usize,

    pat: Tables,
    pmt: Tables,
    eit: Tables,
    sdt: Tables,

    #[allow(dead_code)]
    nit: Tables,
    #[allow(dead_code)]
    cat: Tables,
    #[allow(dead_code)]
    bat: Tables,

    pmt_pids: PMTPids,

    #[allow(dead_code)]
    streams: Streams,
}

impl Default for Demuxer {
    fn default() -> Self {
        Demuxer {
            offset: 0,

            pat: Default::default(),
            pmt: Default::default(),
            eit: Default::default(),
            sdt: Default::default(),
            nit: Default::default(),
            cat: Default::default(),
            bat: Default::default(),

            pmt_pids: Default::default(),

            streams: Default::default(),
        }
    }
}

unsafe impl Send for Demuxer {}

impl Demuxer {
    pub fn new() -> Demuxer {
        Default::default()
    }

    /// mutably borrow a reference to the underlying tables
    /// by pid
    fn with_tables_mut<F>(&mut self, pid_or_pmt: (PID, bool), f: F) -> Result<()>
    where
        F: Fn(&mut Tables) -> Result<()>,
    {
        f(match pid_or_pmt {
            (PID::PAT, false) => &mut self.pat,
            (PID::SDT, false) => &mut self.sdt,
            (PID::EIT, false) => &mut self.eit,
            (PID::NIT, false) => &mut self.nit,
            (PID::CAT, false) => &mut self.cat,
            (_, true) => &mut self.pmt,
            _ => unreachable!(),
        })
    }

    // TODO: move to macros?
    #[inline(always)]
    fn demux_section(&mut self, pid_or_pmt: (PID, bool), pkt: &Packet) -> Result<()> {
        let buf = pkt.buf_payload_section()?;

        self.with_tables_mut(pid_or_pmt, |tables| {
            if pkt.pusi() {
                let (id, section_number) = match pid_or_pmt {
                    (PID::PAT, false) => {
                        let s = PAT::try_new(buf)?;
                        (s.subtable_id(), s.section_number())
                    }
                    (PID::SDT, false) => {
                        let s = SDT::try_new(buf)?;
                        (s.subtable_id(), s.section_number())
                    }
                    (PID::EIT, false) => {
                        let s = EIT::try_new(buf)?;
                        (s.subtable_id(), s.section_number())
                    }
                    (_, true) => {
                        let s = PMT::try_new(buf)?;
                        (s.subtable_id(), s.section_number())
                    }
                    _ => unreachable!(),
                };

                let table = tables.map.entry(id).or_insert_with(|| Table::new(id));

                let section_ref = match table.sections.get_mut(section_number) {
                    Some(section_ref) => {
                        {
                            let mut section = (*section_ref).borrow_mut();

                            {
                                let raw = section.buf.0.get_ref().as_slice();

                                match pid_or_pmt {
                                    (PID::PAT, false) => println!("{:?}", PAT::new(raw)),
                                    (PID::SDT, false) => println!("{:?}", SDT::new(raw)),
                                    (PID::EIT, false) => println!("{:?}", EIT::new(raw)),
                                    (_, true) => println!("{:?}", PMT::new(raw)),
                                    _ => {}
                                };
                            }

                            section.buf.reset();
                        }

                        section_ref.clone()
                    }
                    None => {
                        let section_ref = Section::new(section_number).into_ref();

                        table.sections.push(section_ref.clone());

                        section_ref
                    }
                };

                tables.current = Some(section_ref);
            }

            if let Some(section_ref) = &tables.current {
                let mut section = (*section_ref).borrow_mut();
                section.buf.0.write_all(buf)?;
            }

            Ok(())
        })?;

        Ok(())
    }

    pub fn demux(&mut self, raw: &[u8]) -> Result<()> {
        self.offset += raw.len();

        let pkt = Packet::new(&raw)?;
        let pid = pkt.pid();

        if pid.is_null() {
            return Ok(());
        }

        match pid {
            PID::PAT => {
                self.demux_section((pid, false), &pkt)?;

                // extract pids from PAT
                if self.pmt_pids.0.is_empty() {
                    let buf = pkt.buf_payload_section()?;
                    let pat = PAT::new(buf);

                    // TODO: refactor via iter/to-iter
                    for pid in pat
                        .programs()
                        .filter_map(Result::ok)
                        .filter(|p| p.pid().is_program_map())
                        .map(|p| PID::from(p.pid()))
                    {
                        self.pmt_pids.push_uniq(pid)
                    }
                }
            }
            PID::SDT | PID::EIT /* | PID::NIT | PID::CAT | PID::BAT */ =>
                self.demux_section((pid, false), &pkt)?,
            PID::Other(..) => {
                // PAT not ready yet
                if self.pmt_pids.0.is_empty() {
                    return Ok(());
                }

                if !self.pmt_pids.0.is_empty() && self.pmt_pids.has(pid) {
                    self.demux_section((pid, true), &pkt)?;
                    // TODO: generate streams
                } else {
                    let buf = pkt.buf_payload_pes()?;

                    let stream = self
                        .streams
                        .map
                        .entry(pid)
                        .or_insert_with(|| Stream::new(pid));

                    if pkt.pusi() {
                        if !stream.buf.is_empty() {
                            let sz = stream.buf.0.position();

                            println!(
                                "(0x{:016X}) :pid {:?} :pts {:?} :dts {:?} :sz {}",
                                stream.offset,
                                stream.pid,
                                stream.pts.map(DurationFmt::from),
                                stream.dts.map(DurationFmt::from),
                                sz,
                            );
                        }

                        stream.buf.reset();

                        let pes = PES::new(buf);

                        stream.offset += self.offset + raw.len() - buf.len();
                        stream.pts = pes.pts().map(Duration::from);
                        stream.dts = pes.dts().map(Duration::from);
                        stream.buf.0.write_all(pes.buf_seek_payload())?;
                    } else {
                        stream.buf.0.write_all(buf)?
                    }
                }
            }
            _ => return Ok(()),
        }

        Ok(())
    }
}
