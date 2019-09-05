use std::fmt;
use std::time::Duration;

use crate::duration_fmt::DurationFmt;
use crate::error::{Error, Kind as ErrorKind};
use crate::rational;
use crate::rational::Rational;
use crate::result::Result;

/// Program clock reference,
/// stored as 33 bits base, 6 bits reserved, 9 bits extension.
/// The value is calculated as base * 300 + extension.
pub struct PCR<'buf> {
    buf: &'buf [u8],
}

impl<'buf> PCR<'buf> {
    pub const SZ: usize = 6;
    const TB: Rational = rational::TB_27MHZ;

    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> PCR<'buf> {
        PCR { buf }
    }

    #[inline(always)]
    #[allow(dead_code)]
    fn try_new(buf: &'buf [u8]) -> Result<PCR<'buf>> {
        let a = Self::new(buf);
        a.validate()?;
        Ok(a)
    }

    #[inline(always)]
    fn validate(&self) -> Result<()> {
        if self.buf.len() < Self::SZ {
            Err(Error::new(ErrorKind::Buf(self.buf.len(), Self::SZ)))
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    fn base(&self) -> u64 {
        (u64::from(self.buf[0]) << 25)
            | (u64::from(self.buf[1]) << 17)
            | (u64::from(self.buf[2]) << 9)
            | (u64::from(self.buf[3]) << 1)
            | u64::from((self.buf[4] & 0b1000_0000) >> 7)
    }

    #[inline(always)]
    fn ext(&self) -> u16 {
        (u16::from(self.buf[4] & 0b0000_00001) << 8) | u16::from(self.buf[5])
    }

    /// 27MHz
    pub fn value(&self) -> u64 {
        self.base() * 300 + u64::from(self.ext())
    }

    /// nanoseconds
    pub fn ns(&self) -> u64 {
        rational::rescale(self.value(), Self::TB, rational::TB_1NS)
    }
}

impl<'buf> From<&PCR<'buf>> for Duration {
    fn from(pcr: &PCR) -> Self {
        Duration::from_nanos(pcr.ns())
    }
}

impl<'buf> From<&PCR<'buf>> for DurationFmt {
    fn from(pcr: &PCR) -> Self {
        DurationFmt::from_nanos(pcr.ns())
    }
}

impl<'buf> fmt::Debug for PCR<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ":pcr (:raw {:08X}:{:04X} :v(27MHz) {} :duration {})",
            self.base(),
            self.ext(),
            self.value(),
            DurationFmt::from(self)
        )
    }
}

impl<'buf> fmt::Display for PCR<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":pcr {}", DurationFmt::from(self))
    }
}
