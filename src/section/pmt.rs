use std::fmt;

use crate::descriptor::Descriptor;
use crate::result::Result;
use crate::stream_type::StreamType;
use crate::subtable_id::{SubtableID, SubtableIDer};

use super::traits::*;

/// ISO/IEC 13818-1
///
/// Program Map Table
pub struct PMT<'buf> {
    buf: &'buf [u8],
}

impl<'buf> PMT<'buf> {
    const HEADER_SPECIFIC_SZ: usize = 4;
    const HEADER_FULL_SZ: usize = HEADER_SZ + SYNTAX_SECTION_SZ + Self::HEADER_SPECIFIC_SZ;

    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> PMT<'buf> {
        PMT { buf }
    }

    #[inline(always)]
    pub fn try_new(buf: &'buf [u8]) -> Result<PMT<'buf>> {
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
        let lft = Self::HEADER_FULL_SZ + (self.program_info_length() as usize);
        let mut rght = HEADER_SZ + (self.section_length() as usize);

        if rght >= self.buf.len() {
            rght = self.buf.len();
        }

        rght -= CRC32_SZ;

        &self.buf[lft..rght]
    }

    /// seek
    #[inline(always)]
    fn buf_descriptors(&self) -> &'buf [u8] {
        let lft = Self::HEADER_FULL_SZ;
        let rght = Self::HEADER_FULL_SZ + (self.program_info_length() as usize);

        &self.buf[lft..rght]
    }

    #[inline(always)]
    pub fn descriptors(&self) -> Option<Cursor<'buf, Descriptor>> {
        if self.program_info_length() != 0 {
            Some(Cursor::new(self.buf_descriptors()))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn streams(&self) -> Cursor<'buf, Stream> {
        Cursor::new(self.buf_streams())
    }

    #[inline(always)]
    pub fn program_number(&self) -> u16 {
        self.table_id_extension()
    }
}

trait WithPMTHeaderSpecific<'buf>: Bufer<'buf> {
    /// buffer seeked
    #[inline(always)]
    fn b(&self) -> &'buf [u8] {
        &self.buf()[HEADER_SZ + SYNTAX_SECTION_SZ..]
    }

    #[inline(always)]
    fn pcr_pid(&self) -> u16 {
        u16::from(self.b()[0] & 0b0001_1111) | u16::from(self.b()[1])
    }

    #[inline(always)]
    fn program_info_length(&self) -> u16 {
        u16::from(self.b()[2] & 0b0000_1111) | u16::from(self.b()[3])
    }
}

impl<'buf> Bufer<'buf> for PMT<'buf> {
    fn buf(&self) -> &'buf [u8] {
        self.buf
    }
}

impl<'buf> WithHeader<'buf> for PMT<'buf> {}
impl<'buf> WithTableIDExtension<'buf> for PMT<'buf> {}
impl<'buf> WithSyntaxSection<'buf> for PMT<'buf> {}
impl<'buf> WithPMTHeaderSpecific<'buf> for PMT<'buf> {}
impl<'buf> WithCRC32<'buf> for PMT<'buf> {}

impl<'buf> fmt::Debug for PMT<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ":PMT (:tid {:?} :tid~p-n(ext)~vn {}~{}~{} :program-number {} :section-length {} :pcr-pid {} :program-info-length {})",
            self.table_id(),
            u8::from(self.table_id()),
            self.program_number(),
            self.version_number(),
            self.program_number(),
            self.section_length(),
            self.pcr_pid(),
            self.program_info_length(),
        )?;

        write!(f, "\n  :descriptors")?;
        match self.descriptors() {
            Some(descs) => {
                for d in descs.filter_map(Result::ok) {
                    write!(f, "\n    ")?;
                    d.fmt(f)?;
                }
            }
            None => write!(f, " ~")?,
        }

        write!(f, "\n  :streams")?;
        for p in self.streams().filter_map(Result::ok) {
            write!(f, "\n    ")?;
            p.fmt(f)?;
        }

        Ok(())
    }
}

impl<'buf> SubtableIDer for PMT<'buf> {
    #[inline(always)]
    fn subtable_id(&self) -> SubtableID {
        SubtableID::PMT(
            self.table_id(),
            self.program_number(),
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
    fn stream_type(&self) -> StreamType {
        StreamType::from(self.buf[0])
    }

    #[inline(always)]
    pub fn pid(&self) -> u16 {
        (u16::from(self.buf[1] & 0b0001_1111) << 8) | u16::from(self.buf[2])
    }

    #[inline(always)]
    fn es_info_length(&self) -> u16 {
        (u16::from(self.buf[3] & 0b0000_1111) << 8) | u16::from(self.buf[4])
    }

    /// seek
    #[inline(always)]
    fn buf_descriptors(&self) -> &'buf [u8] {
        let lft = Self::HEADER_SZ;
        let mut rght = lft + (self.es_info_length() as usize);

        if rght >= self.buf.len() {
            rght = self.buf.len();
        }

        &self.buf[lft..rght]
    }

    #[inline(always)]
    pub fn descriptors(&self) -> Option<Cursor<'buf, Descriptor>> {
        if self.es_info_length() != 0 {
            Some(Cursor::new(self.buf_descriptors()))
        } else {
            None
        }
    }
}

impl<'buf> Szer for Stream<'buf> {
    #[inline(always)]
    fn sz(&self) -> usize {
        Self::HEADER_SZ + (self.es_info_length() as usize)
    }
}

impl<'buf> TryNewer<'buf> for Stream<'buf> {
    #[inline(always)]
    fn try_new(buf: &'buf [u8]) -> Result<Stream<'buf>> {
        let p = Stream::new(buf);
        Ok(p)
    }
}

impl<'buf> fmt::Debug for Stream<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ":stream (:pid {:?} :stream-type {:?})",
            self.pid(),
            self.stream_type()
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
