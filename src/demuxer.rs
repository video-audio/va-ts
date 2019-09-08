use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Cursor, Write};
use std::rc::Rc;
use std::time::Duration;

use crate::packet::Packet as TsPacket;
use crate::pes::PES;
use crate::pid::PID;
use crate::result::Result;
use crate::section::{WithHeader, WithSyntaxSection};
use crate::subtable_id::{SubtableID, SubtableIDer};
use crate::{EIT, PAT, PMT, SDT};

pub struct Buf(pub Cursor<Vec<u8>>);

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
    pub fn sz(&self) -> usize {
        self.0.position() as usize
    }
}

impl Default for Buf {
    fn default() -> Self {
        Buf(Cursor::new(Vec::with_capacity(2048)))
    }
}

pub struct Section {
    /// parent table-id
    table_id: SubtableID,

    /// number inside table sections
    number: u8,

    /// full section size with header, data, CRC
    sz: usize,

    pub buf: Buf,
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

pub struct Sections(pub Vec<SectionRef>);

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

pub struct Table {
    /// mpeg-ts last-section-number
    last_section_number: u8,
    pub sections: Sections,
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
                let first = (&self.sections.0[0]).borrow();

                first.number == 0
                    && last.number == self.last_section_number
                    && first.done()
                    && last.done()
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

pub struct Packet {
    pub pid: PID,

    pub offset: usize,

    /// presentation time stamp
    pub pts: Option<Duration>,

    /// decode time stamp
    pub dts: Option<Duration>,

    pub buf: Buf,

    /// got ts PUSI
    started: bool,
}

impl Packet {
    fn new(pid: PID) -> Packet {
        Packet {
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
struct Packets(HashMap<PID, Packet>);

/// pid, packet-constructed
#[derive(Debug)]
struct PMTPids(Vec<(PID, bool)>);

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

    /// got pid? and pid is parsed and packet builded
    #[inline(always)]
    fn is_packet_builded(&self, pid: PID) -> Option<bool> {
        self.0.iter().find(|p| p.0 == pid).map(|p| p.1)
    }

    #[inline(always)]
    fn set_is_packet_builded(&mut self, pid: PID, v: bool) {
        if let Some(p) = self.0.iter_mut().find(|p| p.0 == pid) {
            p.1 = v;
        }
    }

    /// all pids are parsed?
    #[inline(always)]
    #[allow(dead_code)]
    fn are_all_packets_builded(&self) -> bool {
        !self.0.iter().any(|p| !(*p).1)
    }
}

impl Default for PMTPids {
    fn default() -> Self {
        PMTPids(Vec::with_capacity(3))
    }
}

pub trait DemuxerEvents {
    fn on_table(&mut self, _: SubtableID, _: &Table) {}
    fn on_packet(&mut self, _: &Packet) {}
}

/// TODO: use tree, redix tree here
/// TODO: add benches
pub struct Demuxer<T>
where
    T: DemuxerEvents,
{
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

    packets: Packets,

    // TODO: add PID with state(is-parsed or not)
    //       for multiple PMTs
    pmt_pids: PMTPids,

    events: T,
}

unsafe impl<T> Send for Demuxer<T> where T: DemuxerEvents {}

impl<T> Demuxer<T>
where
    T: DemuxerEvents,
{
    pub fn new(events: T) -> Demuxer<T> {
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

            packets: Default::default(),

            events,
        }
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

    /// build packets cache
    // TODO: also do via iterator
    // TODO: .iter().collect() for lazy collection
    #[inline(always)]
    fn build_packets(&mut self) {
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
                    self.packets
                        .0
                        .entry(pid)
                        .or_insert_with(|| Packet::new(pid));
                }
            }
        }
    }

    // TODO: move to macros?
    #[inline(always)]
    fn demux_section(&mut self, pid_or_pmt: (PID, bool), pkt: &TsPacket) -> Result<()> {
        let tables = match pid_or_pmt {
            (PID::PAT, false) => &mut self.pat,
            (PID::SDT, false) => &mut self.sdt,
            (PID::EIT, false) => &mut self.eit,
            (PID::NIT, false) => &mut self.nit,
            (PID::CAT, false) => &mut self.cat,
            (_, true) => &mut self.pmt,
            _ => unreachable!(),
        };

        let buf = pkt.buf_payload_section()?;

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
                            // emit
                            self.events.on_table(section.table_id, &table);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn demux(&mut self, raw: &[u8]) -> Result<()> {
        if self.demux_tables(raw)? {
            return Ok(());
        }

        self.demux_packets(raw)
    }

    /// ffmpeg::avformat_open_input analog
    /// probe input
    /// return: is pid handled?
    pub fn demux_tables(&mut self, raw: &[u8]) -> Result<(bool)> {
        self.offset += raw.len();

        let pkt = TsPacket::new(&raw)?;
        let pid = pkt.pid();

        if pid.is_null() {
            // null packet PID
            return Ok(true);
        }

        match pid {
            PID::PAT => {
                self.demux_section((pid, false), &pkt)?;

                // extract pids from PAT
                if self.pmt_pids.0.is_empty() {
                    self.pmt_pids.0.clear();
                    self.packets.0.clear();
                    self.build_pmt_pids();
                }
            }
            PID::SDT | PID::EIT /* | PID::NIT | PID::CAT | PID::BAT */ =>
                self.demux_section((pid, false), &pkt)?,

            PID::Other(..) => {
                // PAT not ready yet
                // wait for PAT
                if self.pmt_pids.0.is_empty() {
                    return Ok(true);
                }

                match self.pmt_pids.is_packet_builded(pid) {
                    Some(true) => { // got PMT and already builded
                        self.demux_section((pid, true), &pkt)?;
                    },
                    Some(false) => { // got PMT and not builded
                        self.demux_section((pid, true), &pkt)?;

                        self.build_packets();

                        self.pmt_pids.set_is_packet_builded(pid, true);
                    },
                    None => {return Ok(false); }
                }
            }
            _ => {}
        }

        Ok(true)
    }

    /// ffmpeg::av_read_frame analog
    pub fn demux_packets(&mut self, raw: &[u8]) -> Result<()> {
        self.offset += raw.len();

        let pkt = TsPacket::new(&raw)?;
        let pid = pkt.pid();

        if pid.is_null() // null packet PID
        && !pid.is_other() // PID is section/table PID
        // PAT not ready yet
        // wait for pat
        && !self.pmt_pids.0.is_empty()
        {
            return Ok(());
        }

        let mut packet = match self.packets.0.get_mut(&pid) {
            Some(packet) => packet,
            None => return Ok(()), // packet is not builder - wait fot PMT
        };

        let mut buf = pkt.buf_payload_pes()?;

        if pkt.pusi() {
            let pes = PES::new(buf);

            if !packet.buf.is_empty() {
                // emit
                self.events.on_packet(packet);
            }

            packet.buf.reset();
            packet.started = true;
            packet.offset += self.offset + raw.len() - buf.len();
            packet.pts = pes.pts().map(Duration::from);
            packet.dts = pes.dts().map(Duration::from);

            buf = pes.buf_seek_payload();
        }

        if packet.started {
            packet.buf.0.write_all(buf)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
