use crate::error::{Error, Kind as ErrorKind};
use crate::pcr::PCR;
use crate::pid::PID;
use crate::result::Result;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransportScramblingControl {
    NotScrambled,
    ScrambledReserved,
    ScrambledEven,
    ScrambledOdd,
}

impl From<u8> for TransportScramblingControl {
    #[inline(always)]
    fn from(d: u8) -> Self {
        match d {
            0b00 => TransportScramblingControl::NotScrambled,
            0b01 => TransportScramblingControl::ScrambledReserved,
            0b10 => TransportScramblingControl::ScrambledEven,
            0b11 => TransportScramblingControl::ScrambledOdd,

            _ => TransportScramblingControl::NotScrambled,
        }
    }
}

pub struct Adaptation<'buf> {
    buf: &'buf [u8],
}

impl<'buf> Adaptation<'buf> {
    const HEADER_SZ: usize = 1;
    const HEADER_FULL_SZ: usize = 2;

    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> Adaptation<'buf> {
        Adaptation { buf }
    }

    #[inline(always)]
    pub fn try_new(buf: &'buf [u8]) -> Result<Adaptation<'buf>> {
        let a = Self::new(buf);
        a.validate()?;
        Ok(a)
    }

    #[inline(always)]
    fn validate(&self) -> Result<()> {
        if self.buf.len() < Self::HEADER_SZ {
            Err(Error::new(ErrorKind::Buf(self.buf.len(), Self::HEADER_SZ)))
        } else if self.buf.len() < self.sz() {
            Err(Error::new(ErrorKind::Buf(self.buf.len(), self.sz())))
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    pub fn sz(&self) -> usize {
        Self::HEADER_SZ + self.field_length()
    }

    #[inline(always)]
    fn field_length(&self) -> usize {
        self.buf[0] as usize
    }

    #[inline(always)]
    #[allow(dead_code)]
    fn discontinuity_indicator(&self) -> bool {
        (self.buf[1] & 0b1000_0000) != 0
    }

    #[inline(always)]
    #[allow(dead_code)]
    fn random_access_indicator(&self) -> bool {
        (self.buf[1] & 0b0100_0000) != 0
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn elementary_stream_priority_indicator(&self) -> bool {
        (self.buf[1] & 0b0010_0000) != 0
    }

    /// PCR field is present?
    #[inline(always)]
    #[allow(dead_code)]
    fn pcr_flag(&self) -> bool {
        (self.buf[1] & 0b0001_0000) != 0
    }

    /// OPCR field is present?
    #[inline(always)]
    #[allow(dead_code)]
    pub fn opcr_flag(&self) -> bool {
        (self.buf[1] & 0b0000_1000) != 0
    }

    /// splice countdown field is present?
    #[inline(always)]
    #[allow(dead_code)]
    pub fn splicing_point_flag(&self) -> bool {
        (self.buf[1] & 0b0000_0100) != 0
    }

    /// transport private data is present?
    #[inline(always)]
    #[allow(dead_code)]
    pub fn transport_private_data_flag(&self) -> bool {
        (self.buf[1] & 0b0000_0010) != 0
    }

    /// transport private data is present?
    #[inline(always)]
    #[allow(dead_code)]
    pub fn adaptation_field_extension_flag(&self) -> bool {
        (self.buf[1] & 0b0000_0001) != 0
    }

    /// seek to PCR start position
    #[inline(always)]
    #[allow(dead_code)]
    fn buf_seek_pcr(&self) -> &'buf [u8] {
        &self.buf[Self::HEADER_FULL_SZ..]
    }

    /// seek to OPCR start position
    #[inline(always)]
    #[allow(dead_code)]
    fn buf_seek_opcr(&self) -> &'buf [u8] {
        let mut buf = self.buf_seek_pcr();
        if self.pcr_flag() {
            buf = &buf[PCR::SZ..];
        }
        buf
    }

    #[inline(always)]
    pub fn pcr(&self) -> Option<PCR<'buf>> {
        if self.pcr_flag() {
            Some(PCR::new(self.buf_seek_pcr()))
        } else {
            None
        }
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn opcr(&self) -> Option<PCR> {
        if self.opcr_flag() {
            Some(PCR::new(self.buf_seek_opcr()))
        } else {
            None
        }
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn splice_countdown(&self) -> Option<u8> {
        if self.splicing_point_flag() {
            // TODO: implement
            unimplemented!()
        } else {
            None
        }
    }
}

pub struct Header<'buf> {
    buf: &'buf [u8],
}

impl<'buf> Header<'buf> {
    pub const SZ: usize = 4;

    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> Header<'buf> {
        Header { buf }
    }

    #[inline(always)]
    #[allow(dead_code)]
    fn tei(&self) -> bool {
        (self.buf[1] & 0b1000_0000) != 0
    }

    #[inline(always)]
    pub fn pusi(&self) -> bool {
        (self.buf[1] & 0b0100_0000) != 0
    }

    #[inline(always)]
    #[allow(dead_code)]
    fn transport_priority(&self) -> bool {
        (self.buf[1] & 0b0010_0000) != 0
    }

    /// Packet Identifier, describing the payload data.
    #[inline(always)]
    pub fn pid(&self) -> PID {
        PID::from((u16::from(self.buf[1] & 0b0001_1111) << 8) | u16::from(self.buf[2]))
    }

    /// transport-scrambling-control
    #[inline(always)]
    #[allow(dead_code)]
    fn tsc(&self) -> TransportScramblingControl {
        TransportScramblingControl::from((self.buf[3] & 0b1100_0000) >> 6)
    }

    #[inline(always)]
    pub fn got_adaptation(&self) -> bool {
        (self.buf[3] & 0b0010_0000) != 0
    }

    #[inline(always)]
    pub fn got_payload(&self) -> bool {
        (self.buf[3] & 0b0001_0000) != 0
    }

    #[inline(always)]
    pub fn cc(&self) -> u8 {
        self.buf[3] & 0b0000_1111
    }
}
