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
use crate::section::{WithHeader, WithSyntaxSection};
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

    #[inline(always)]
    fn sz(&self) -> usize {
        self.0.position() as usize
    }
}

impl Default for Buf {
    fn default() -> Self {
        Buf(Cursor::new(Vec::with_capacity(2048)))
    }
}

struct Section {
    /// parent table-id
    table_id: SubtableID,

    /// number inside table sections
    number: u8,

    /// full section size with header, data, CRC
    sz: usize,

    buf: Buf,
}

impl Section {
    fn new(table_id: SubtableID, number: u8, sz: usize) -> Section {
        Section {
            table_id,
            number,
            sz,
            buf: Default::default(),
        }
    }

    #[inline(always)]
    fn into_ref(self) -> SectionRef {
        Rc::new(RefCell::new(Box::new(self)))
    }

    /// section consumed all data
    #[inline(always)]
    fn done(&self) -> bool {
        self.sz_need() == 0
    }

    /// sz need to read
    #[inline(always)]
    fn sz_need(&self) -> usize {
        self.sz - self.buf.sz()
    }
}

type SectionRef = Rc<RefCell<Box<Section>>>;

struct Sections(Vec<SectionRef>);

impl Sections {
    #[inline(always)]
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
    /// mpeg-ts last-section-number
    last_section_number: u8,
    sections: Sections,
}

impl Table {
    fn new(last_section_number: u8) -> Table {
        Table {
            last_section_number,
            sections: Default::default(),
        }
    }

    #[inline(always)]
    fn done(&self) -> bool {
        match self.sections.0.len() {
            0 => false,
            n => {
                let last = (&self.sections.0[n - 1]).borrow();
                last.number == self.last_section_number && last.done()
            }
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

    buf: Buf,

    /// got ts PUSI
    started: bool,
}

impl Stream {
    fn new(pid: PID) -> Stream {
        Stream {
            pid,
            offset: 0,
            pts: None,
            dts: None,
            buf: Default::default(),
            started: false,
        }
    }
}

#[derive(Default)]
struct Streams(HashMap<PID, Stream>);

/// ((pid, packet-constructed), all-packets-constructed)
#[derive(Debug)]
struct PMTPids(Vec<(PID, bool)>, bool);

impl PMTPids {
    #[inline(always)]
    fn has(&self, pid: PID) -> bool {
        self.0.iter().any(|p| (*p).0 == pid)
    }

    #[inline(always)]
    fn push_uniq(&mut self, pid: PID) {
        if !self.has(pid) {
            self.0.push((pid, false))
        }
    }
}

impl Default for PMTPids {
    fn default() -> Self {
        PMTPids(Vec::with_capacity(3), false)
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

    // TODO: add PID with state(is-parsed or not)
    //       for multiple PMTs
    pmt_pids: PMTPids,

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

    /// cache pmt pids
    // TODO: also do via iterator
    // TODO: .iter().collect() for lazy collection
    #[inline(always)]
    fn build_pmt_pids(&mut self) {
        for (_, table) in self.pat.map.iter() {
            for section_ref in table.sections.0.iter() {
                let section = (*section_ref).borrow();
                let raw = section.buf.0.get_ref().as_slice();
                let pat = PAT::new(raw);

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
    }

    /// cache streams
    // TODO: also do via iterator
    // TODO: .iter().collect() for lazy collection
    #[inline(always)]
    fn build_streams(&mut self, _pid: PID) {
        for (_, table) in self.pmt.map.iter() {
            for section_ref in table.sections.0.iter() {
                let section = (*section_ref).borrow();
                let raw = section.buf.0.get_ref().as_slice();
                let pmt = PMT::new(raw);

                // TODO: refactor via iter/to-iter
                for pid in pmt
                    .streams()
                    .filter_map(Result::ok)
                    .map(|s| PID::from(s.pid()))
                {
                    self.streams
                        .0
                        .entry(pid)
                        .or_insert_with(|| Stream::new(pid));
                }
            }
        }
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
                let (id, sz, section_number, last_section_number) = match pid_or_pmt {
                    (PID::PAT, false) => {
                        let s = PAT::try_new(buf)?;
                        (
                            s.subtable_id(),
                            s.sz(),
                            s.section_number(),
                            s.last_section_number(),
                        )
                    }
                    (PID::SDT, false) => {
                        let s = SDT::try_new(buf)?;
                        (
                            s.subtable_id(),
                            s.sz(),
                            s.section_number(),
                            s.last_section_number(),
                        )
                    }
                    (PID::EIT, false) => {
                        let s = EIT::try_new(buf)?;
                        (
                            s.subtable_id(),
                            s.sz(),
                            s.section_number(),
                            s.last_section_number(),
                        )
                    }
                    (_, true) => {
                        let s = PMT::try_new(buf)?;
                        (
                            s.subtable_id(),
                            s.sz(),
                            s.section_number(),
                            s.last_section_number(),
                        )
                    }
                    _ => unreachable!(),
                };

                let table = tables
                    .map
                    .entry(id)
                    .or_insert_with(|| Table::new(last_section_number));

                let section_ref = match table.sections.get_mut(section_number) {
                    Some(section_ref) => {
                        let mut section = (*section_ref).borrow_mut();
                        section.buf.reset();
                        section.sz = sz;

                        section_ref.clone()
                    }
                    None => {
                        let section_ref = Section::new(id, section_number, sz).into_ref();
                        table.sections.push(section_ref.clone());
                        section_ref
                    }
                };

                tables.current = Some(section_ref);
            }

            if let Some(section_ref) = &tables.current {
                {
                    let mut section = (*section_ref).borrow_mut();
                    let sz_need = section.sz_need();

                    // remove null/padding bytes
                    let buf = if buf.len() > sz_need {
                        &buf[0..sz_need]
                    } else {
                        buf
                    };

                    section.buf.0.write_all(buf)?;
                }

                {
                    let section = (*section_ref).borrow();
                    if section.done() {
                        if let Some(table) = tables.map.get(&section.table_id) {
                            if table.done() {
                                // can emit demuxed table here;

                                for section_ref in table.sections.0.iter() {
                                    let section = (*section_ref).borrow();
                                    let raw = section.buf.0.get_ref().as_slice();

                                    match pid_or_pmt {
                                        (PID::PAT, false) => println!("{:?}", PAT::new(raw)),
                                        (PID::SDT, false) => println!("{:?}", SDT::new(raw)),
                                        (PID::EIT, false) => println!("{:?}", EIT::new(raw)),
                                        (_, true) => println!("{:?}", PMT::new(raw)),
                                        _ => {}
                                    };
                                }
                            }
                        }
                    }
                }
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
                    self.pmt_pids.0.clear();
                    self.streams.0.clear();
                    self.build_pmt_pids();
                }
            }
            PID::SDT | PID::EIT /* | PID::NIT | PID::CAT | PID::BAT */ =>
                self.demux_section((pid, false), &pkt)?,

            PID::Other(..) => {
                // PAT not ready yet
                if self.pmt_pids.0.is_empty() {
                    return Ok(());
                }

                if self.pmt_pids.has(pid) {
                    self.demux_section((pid, true), &pkt)?;

                    if self.streams.0.is_empty() {
                        self.build_streams(pid);
                    }

                } else {
                    let mut buf = pkt.buf_payload_pes()?;

                    let mut stream = match self.streams.0.get_mut(&pid) {
                        Some(stream) => stream,
                        None => return Ok(()),
                    };

                    if pkt.pusi() {
                        let pes = PES::new(buf);

                        if !stream.buf.is_empty() {
                            // can emit demuxed stream here;
                            println!(
                                "(0x{:016X}) :pid {:?} :pts {:?} :dts {:?} :sz {}",
                                stream.offset,
                                stream.pid,
                                stream.pts.map(DurationFmt::from),
                                stream.dts.map(DurationFmt::from),
                                stream.buf.sz(),
                            );
                        }

                        stream.buf.reset();
                        stream.started = true;
                        stream.offset += self.offset + raw.len() - buf.len();
                        stream.pts = pes.pts().map(Duration::from);
                        stream.dts = pes.dts().map(Duration::from);

                        buf = pes.buf_seek_payload();
                    }

                    if stream.started {
                        stream.buf.0.write_all(buf)?;
                    }
                }
            }
            _ => return Ok(()),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
