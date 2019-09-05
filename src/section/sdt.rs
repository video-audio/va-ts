use std::fmt;

use crate::descriptor::Descriptor;
use crate::result::Result;
use crate::subtable_id::{SubtableID, SubtableIDer};

use super::traits::*;

/// ETSI EN 300 468 V1.15.1
///
/// Service Description Table
pub struct SDT<'buf> {
    buf: &'buf [u8],
}

impl<'buf> SDT<'buf> {
    const HEADER_SPECIFIC_SZ: usize = 3;
    const HEADER_FULL_SZ: usize = HEADER_SZ + SYNTAX_SECTION_SZ + Self::HEADER_SPECIFIC_SZ;

    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> SDT<'buf> {
        SDT { buf }
    }

    #[inline(always)]
    pub fn try_new(buf: &'buf [u8]) -> Result<SDT<'buf>> {
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
    fn buf_streams(&self) -> &'buf [u8] {
        let lft = Self::HEADER_FULL_SZ;
        let mut rght = HEADER_SZ + (self.section_length() as usize);

        if rght >= self.buf.len() {
            rght = self.buf.len();
        }

        rght -= CRC32_SZ;

        &self.buf[lft..rght]
    }

    #[inline(always)]
    pub fn streams(&self) -> Cursor<'buf, Stream> {
        Cursor::new(self.buf_streams())
    }

    #[inline(always)]
    pub fn transport_stream_id(&self) -> u16 {
        self.table_id_extension()
    }
}

trait WithSDTHeaderSpecific<'buf>: Bufer<'buf> {
    /// buffer seeked
    #[inline(always)]
    fn b(&self) -> &'buf [u8] {
        &self.buf()[HEADER_SZ + SYNTAX_SECTION_SZ..]
    }

    #[inline(always)]
    fn original_network_id(&self) -> u16 {
        u16::from(self.b()[0]) | u16::from(self.b()[1])
    }
}

impl<'buf> Bufer<'buf> for SDT<'buf> {
    fn buf(&self) -> &'buf [u8] {
        self.buf
    }
}

impl<'buf> WithHeader<'buf> for SDT<'buf> {}
impl<'buf> WithTableIDExtension<'buf> for SDT<'buf> {}
impl<'buf> WithSyntaxSection<'buf> for SDT<'buf> {}
impl<'buf> WithSDTHeaderSpecific<'buf> for SDT<'buf> {}
impl<'buf> WithCRC32<'buf> for SDT<'buf> {}

impl<'buf> fmt::Debug for SDT<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ":SDT (:id {:?} :transport-stream-id {} :section-length {} :section {}/{})",
            self.subtable_id(),
            self.transport_stream_id(),
            self.section_length(),
            self.section_number(),
            self.last_section_number(),
        )?;

        write!(f, "\n  :streams")?;
        for p in self.streams().filter_map(Result::ok) {
            write!(f, "\n    ")?;
            p.fmt(f)?;
        }

        Ok(())
    }
}

impl<'buf> SubtableIDer for SDT<'buf> {
    #[inline(always)]
    fn subtable_id(&self) -> SubtableID {
        SubtableID::SDT(
            self.table_id(),
            self.transport_stream_id(),
            self.original_network_id(),
            self.version_number(),
        )
    }
}

pub struct Stream<'buf> {
    buf: &'buf [u8],
}

impl<'buf> Stream<'buf> {
    const HEADER_SZ: usize = 5;

    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> Stream<'buf> {
        Stream { buf }
    }

    #[inline(always)]
    pub fn service_id(&self) -> u16 {
        u16::from(self.buf[0]) | u16::from(self.buf[1])
    }

    #[inline(always)]
    pub fn eit_schedule_flag(&self) -> bool {
        ((self.buf[2] & 0b0000_0010) >> 1) != 0
    }

    #[inline(always)]
    pub fn eit_present_following_flag(&self) -> bool {
        (self.buf[2] & 0b0000_0001) != 0
    }

    // TODO: add enum
    #[inline(always)]
    pub fn running_status(&self) -> u8 {
        (self.buf[3] & 0b1110_0000) >> 5
    }

    #[inline(always)]
    pub fn free_ca_mode(&self) -> bool {
        (self.buf[3] & 0b0001_0000) != 0
    }

    #[inline(always)]
    pub fn descriptors_loop_length(&self) -> u16 {
        (u16::from(self.buf[3] & 0b0000_1111) << 8) | u16::from(self.buf[4])
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
}

impl<'buf> Szer for Stream<'buf> {
    #[inline(always)]
    fn sz(&self) -> usize {
        Self::HEADER_SZ + (self.descriptors_loop_length() as usize)
    }
}

impl<'buf> TryNewer<'buf> for Stream<'buf> {
    #[inline(always)]
    fn try_new(buf: &'buf [u8]) -> Result<Stream<'buf>> {
        let s = Stream::new(buf);
        Ok(s)
    }
}

impl<'buf> fmt::Debug for Stream<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ":stream (:service-id {:?} :running-status {:?})",
            self.service_id(),
            self.running_status(),
        )?;

        write!(f, "\n      :descriptors")?;
        match self.descriptors() {
            Some(descs) => {
                for d in descs.filter_map(Result::ok) {
                    write!(f, "\n        ")?;
                    d.fmt(f)?;
                }
            }
            None => write!(f, " ~")?,
        }

        Ok(())
    }
}
