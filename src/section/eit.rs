use std::fmt;
use std::time::Duration;

use chrono::prelude::*;

use crate::annex_c;
use crate::descriptor::Descriptor;
use crate::duration_fmt::DurationFmt;
use crate::error::{Error, Kind as ErrorKind};
use crate::result::Result;
use crate::subtable_id::{SubtableID, SubtableIDer};

use super::traits::*;

/// ETSI EN 300 468 V1.15.1
///
/// Event Information Table
pub struct EIT<'buf> {
    buf: &'buf [u8],
}

impl<'buf> EIT<'buf> {
    const HEADER_SPECIFIC_SZ: usize = 6;
    const HEADER_FULL_SZ: usize = HEADER_SZ + SYNTAX_SECTION_SZ + Self::HEADER_SPECIFIC_SZ;

    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> EIT<'buf> {
        EIT { buf }
    }

    #[inline(always)]
    pub fn try_new(buf: &'buf [u8]) -> Result<EIT<'buf>> {
        let s = Self::new(buf);
        s.validate()?;
        Ok(s)
    }

    #[inline(always)]
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }

    /// seek
    #[inline(always)]
    fn buf_events(&self) -> &'buf [u8] {
        let lft = Self::HEADER_FULL_SZ;
        let mut rght = HEADER_SZ + (self.section_length() as usize);

        if rght >= self.buf.len() {
            rght = self.buf.len();
        }

        rght -= CRC32_SZ;

        &self.buf[lft..rght]
    }

    #[inline(always)]
    pub fn events(&self) -> Cursor<'buf, Event> {
        Cursor::new(self.buf_events())
    }

    #[inline(always)]
    pub fn service_id(&self) -> u16 {
        self.table_id_extension()
    }
}

trait WithEITHeaderSpecific<'buf>: Bufer<'buf> {
    /// buffer seeked
    #[inline(always)]
    fn b(&self) -> &'buf [u8] {
        &self.buf()[HEADER_SZ + SYNTAX_SECTION_SZ..]
    }

    #[inline(always)]
    fn transport_stream_id(&self) -> u16 {
        u16::from(self.b()[0]) | u16::from(self.b()[1])
    }

    #[inline(always)]
    fn original_network_id(&self) -> u16 {
        u16::from(self.b()[2]) | u16::from(self.b()[3])
    }

    #[inline(always)]
    fn segment_last_section_number(&self) -> u8 {
        self.b()[4]
    }

    #[inline(always)]
    fn last_table_id(&self) -> u8 {
        self.b()[5]
    }
}

impl<'buf> Bufer<'buf> for EIT<'buf> {
    fn buf(&self) -> &'buf [u8] {
        self.buf
    }
}

impl<'buf> WithHeader<'buf> for EIT<'buf> {}
impl<'buf> WithTableIDExtension<'buf> for EIT<'buf> {}
impl<'buf> WithSyntaxSection<'buf> for EIT<'buf> {}
impl<'buf> WithEITHeaderSpecific<'buf> for EIT<'buf> {}
impl<'buf> WithCRC32<'buf> for EIT<'buf> {}

impl<'buf> fmt::Debug for EIT<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ":EIT (:id {:?} :section-length {:3} :section {}/{})",
            self.subtable_id(),
            self.section_length(),
            self.section_number(),
            self.last_section_number(),
        )?;

        write!(f, "\n  :events")?;
        for rese in self.events() {
            write!(f, "\n    ")?;
            match rese {
                Ok(e) => {
                    e.fmt(f)?;
                }
                Err(err) => {
                    write!(f, "error parse EIT event: {}", err)?;
                }
            }
        }

        Ok(())
    }
}

impl<'buf> SubtableIDer for EIT<'buf> {
    #[inline(always)]
    fn subtable_id(&self) -> SubtableID {
        SubtableID::EIT(
            self.table_id(),
            self.service_id(),
            self.transport_stream_id(),
            self.original_network_id(),
            self.version_number(),
        )
    }
}

pub struct Event<'buf> {
    buf: &'buf [u8],
}

impl<'buf> Event<'buf> {
    const HEADER_SZ: usize = 12;

    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> Event<'buf> {
        Event { buf }
    }

    #[inline(always)]
    pub fn validate(&self) -> Result<()> {
        if self.buf.len() < Self::HEADER_SZ {
            Err(Error::new(ErrorKind::Buf(self.buf.len(), Self::HEADER_SZ)))
        } else if self.buf.len() < self.sz() {
            Err(Error::new(ErrorKind::Buf(self.buf.len(), self.sz())))
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    pub fn event_id(&self) -> u16 {
        (u16::from(self.buf[0]) << 8) | u16::from(self.buf[1])
    }

    #[inline(always)]
    pub fn start_time(&self) -> DateTime<Utc> {
        // must
        annex_c::from_bytes_into_date_time_utc(&self.buf[2..7]).unwrap()
    }

    #[inline(always)]
    pub fn duration(&self) -> Duration {
        // must
        annex_c::from_bytes_into_duration(&self.buf[7..10]).unwrap()
    }

    /// seek
    #[inline(always)]
    fn buf_descriptors(&self) -> &'buf [u8] {
        let lft = Self::HEADER_SZ;
        let mut rght = lft + (self.descriptors_loop_length() as usize);

        if rght >= self.buf.len() {
            rght = self.buf.len();
        }

        &self.buf[lft..rght]
    }

    #[inline(always)]
    pub fn descriptors(&self) -> Option<Cursor<'buf, Descriptor>> {
        if self.descriptors_loop_length() != 0 {
            Some(Cursor::new(self.buf_descriptors()))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn descriptors_loop_length(&self) -> u16 {
        (u16::from(self.buf[10] & 0b0000_1111) << 8) | u16::from(self.buf[11])
    }
}

impl<'buf> Szer for Event<'buf> {
    #[inline(always)]
    fn sz(&self) -> usize {
        Self::HEADER_SZ + (self.descriptors_loop_length() as usize)
    }
}

impl<'buf> TryNewer<'buf> for Event<'buf> {
    #[inline(always)]
    fn try_new(buf: &'buf [u8]) -> Result<Event<'buf>> {
        let s = Event::new(buf);
        s.validate()?;
        Ok(s)
    }
}

impl<'buf> fmt::Debug for Event<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ":event (:start-time {} :duration {})",
            self.start_time(),
            DurationFmt::from(self.duration()),
        )?;

        write!(f, "\n      :descriptors")?;
        match self.descriptors() {
            Some(descs) => {
                for resd in descs {
                    write!(f, "\n        ")?;
                    match resd {
                        Ok(d) => {
                            d.fmt(f)?;
                        }
                        Err(err) => {
                            write!(f, "error parse descriptor: {}", err)?;
                        }
                    }
                }
            }
            None => write!(f, " ~")?,
        }

        Ok(())
    }
}
