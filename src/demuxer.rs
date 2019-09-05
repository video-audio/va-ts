use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Cursor, Write};
use std::rc::Rc;

use crate::packet::Packet;
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

    #[allow(dead_code)]
    buf: Buf,
}

impl Stream {
    fn new(pid: PID) -> Stream {
        Stream {
            pid: pid,
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
    pat: Tables,
    #[allow(dead_code)]
    pmt: Tables,
    eit: Tables,
    sdt: Tables,

    pmt_pids: PMTPids,

    #[allow(dead_code)]
    streams: Streams,
}

impl Default for Demuxer {
    fn default() -> Self {
        Demuxer {
            pat: Default::default(),
            pmt: Default::default(),
            eit: Default::default(),
            sdt: Default::default(),

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
    fn with_tables_mut<F>(&mut self, pid: PID, f: F) -> Result<()>
    where
        F: Fn(&mut Tables) -> Result<()>,
    {
        f(match pid {
            PID::PAT => &mut self.pat,
            PID::SDT => &mut self.sdt,
            PID::EIT => &mut self.eit,
            _ => unreachable!(),
        })
    }

    // TODO: implement
    fn demux_section() -> Result<()> {
        Ok(())
    }

    pub fn demux(&mut self, raw: &[u8]) -> Result<()> {
        let pkt = Packet::new(&raw)?;
        let pid = pkt.pid();

        if pid.is_null() {
            return Ok(());
        }

        match pid {
            PID::PAT | PID::SDT | PID::EIT /*| PID::NIT | PID::CAT | PID::BAT */ => {
                let buf = pkt.buf_payload_section()?;

                self.with_tables_mut(pid, |tables| {
                    if pkt.pusi() {
                        let (id, section_number) = match pid {
                            PID::PAT => {
                                let s = PAT::try_new(buf)?;
                                (s.subtable_id(), s.section_number())
                            }
                            PID::SDT => {
                                let s = SDT::try_new(buf)?;
                                (s.subtable_id(), s.section_number())
                            }
                            PID::EIT => {
                                let s = EIT::try_new(buf)?;
                                (s.subtable_id(), s.section_number())
                            }
                            _ => unreachable!()
                        };

                        let table = tables.map.entry(id).or_insert_with(|| Table::new(id));

                        let section_ref = match table.sections.get_mut(section_number) {
                            Some(section_ref) => {
                                {
                                    let mut section = (*section_ref).borrow_mut();

                                    {
                                        let raw = section.buf.0.get_ref().as_slice();

                                        match pid {
                                            PID::PAT => println!("{:?}", PAT::new(raw)),
                                            PID::SDT => println!("{:?}", SDT::new(raw)),
                                            PID::EIT => println!("{:?}", EIT::new(raw)),
                                            _ => {},
                                        };
                                    }

                                    section.buf.reset();
                                }

                                section_ref.clone()
                            },
                            None => {
                                let section_ref = Section::new(section_number).into_ref();

                                table.sections.push(section_ref.clone());

                                section_ref
                            },
                        };

                        tables.current = Some(section_ref);
                    }

                    if let Some(section_ref) = &tables.current {
                        let mut section = (*section_ref).borrow_mut();
                        section.buf.0.write_all(buf)?;
                    }

                    Ok(())
                })?;

                // extract pids from PAT
                if pid == PID::PAT && self.pmt_pids.0.is_empty() {
                    let pat = PAT::new(buf);
                    // TODO: move as iterator
                    for pmt_pid in pat.programs().filter_map(Result::ok).filter(|p| p.pid().is_program_map()).map(|p| PID::from(p.pid())) {
                        self.pmt_pids.push_uniq(pmt_pid);
                    }
                }
            }
            PID::Other(..) => {
                // PAT not ready yet
                if self.pmt_pids.0.is_empty() {
                    return Ok(());
                }

                let buf = pkt.buf_payload_section()?;

                if !self.pmt_pids.0.is_empty() && self.pmt_pids.has(pid) {
                    let tables = &mut self.pmt;
                    let s = PMT::try_new(buf)?;
                    let (id, section_number) = (s.subtable_id(), s.section_number());

                    let table = tables.map.entry(id).or_insert_with(|| Table::new(id));

                    let section_ref = match table.sections.get_mut(section_number) {
                        Some(section_ref) => {
                            {
                                let mut section = (*section_ref).borrow_mut();

                                {
                                    let raw = section.buf.0.get_ref().as_slice();
                                    println!("{:?}", PMT::new(raw));
                                }

                                section.buf.reset();
                            }

                            section_ref.clone()
                        },
                        None => {
                            let section_ref = Section::new(section_number).into_ref();

                            table.sections.push(section_ref.clone());

                            section_ref
                        },
                    };

                    {
                        let mut section = (*section_ref).borrow_mut();
                        section.buf.0.write_all(buf)?;
                    }

                    tables.current = Some(section_ref);
                } else {
                    // let stream = self.streams.map.entry(pid).or_insert_with(|| Stream::new(pid));

                    // println!("{:?}", stream.pid)
                }
            }
            _ => return Ok(()),
        }

        Ok(())
    }
}
