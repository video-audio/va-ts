mod tag;

mod desc_0x0a;
mod desc_dvb_0x48;
mod desc_dvb_0x4d;
mod desc_dvb_0x4e;
mod desc_dvb_0x53;
mod desc_dvb_0x54;
mod desc_dvb_0x56;
mod desc_dvb_0x6a;

use std::fmt;
use std::str;

use crate::error::{Error, Kind as ErrorKind};
use crate::result::Result;
use crate::section::{Szer, TryNewer};

pub use self::desc_0x0a::Desc0x0A;
pub use self::desc_dvb_0x48::DescDVB0x48;
pub use self::desc_dvb_0x4d::DescDVB0x4D;
pub use self::desc_dvb_0x4e::DescDVB0x4E;
pub use self::desc_dvb_0x53::DescDVB0x53;
pub use self::desc_dvb_0x54::DescDVB0x54;
pub use self::desc_dvb_0x56::DescDVB0x56;
pub use self::desc_dvb_0x6a::DescDVB0x6A;
pub use self::tag::{Tag, TagDVB};

#[derive(Clone)]
pub struct Descriptor<'buf> {
    buf: &'buf [u8],
}

impl<'buf> Descriptor<'buf> {
    const HEADER_SZ: usize = 2;

    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> Descriptor<'buf> {
        Descriptor { buf }
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
    pub fn tag(&self) -> Tag {
        Tag::from(self.buf[0])
    }

    #[inline(always)]
    pub fn is_dvb_service(&self) -> bool {
        self.tag().is_dvb_service()
    }

    #[inline(always)]
    pub fn is_dvb_short_event(&self) -> bool {
        self.tag().is_dvb_short_event()
    }

    #[inline(always)]
    fn len(&self) -> u8 {
        self.buf[1]
    }

    /// seek
    #[inline(always)]
    pub fn buf_data(&self) -> &'buf [u8] {
        &self.buf[Self::HEADER_SZ..]
    }

    #[inline(always)]
    fn data_as_unicode(&'buf self) -> &'buf str {
        str::from_utf8(&self.buf_data()).unwrap_or("---")
    }
}

impl<'buf> Szer for Descriptor<'buf> {
    #[inline(always)]
    fn sz(&self) -> usize {
        Self::HEADER_SZ + (self.len() as usize)
    }
}

impl<'buf> TryNewer<'buf> for Descriptor<'buf> {
    #[inline(always)]
    fn try_new(buf: &'buf [u8]) -> Result<Descriptor<'buf>> {
        let mut d = Descriptor::new(buf);
        d.validate()?;
        d.buf = &buf[..d.sz()]; // slice
        Ok(d)
    }
}

impl<'buf> fmt::Debug for Descriptor<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":desc (:tag {:?} :length {})", self.tag(), self.len())?;
        write!(f, "\n          ")?;

        match self.tag() {
            Tag::ISO639 => {
                Desc0x0A::new(self.buf_data()).fmt(f)?;
            }
            Tag::DVB(TagDVB::Service) => {
                DescDVB0x48::new(self.buf_data()).fmt(f)?;
            }
            Tag::DVB(TagDVB::ShortEvent) => {
                DescDVB0x4D::new(self.buf_data()).fmt(f)?;
            }
            Tag::DVB(TagDVB::ExtendedEvent) => {
                DescDVB0x4E::new(self.buf_data()).fmt(f)?;
            }
            Tag::DVB(TagDVB::CAIdentifier) => {
                DescDVB0x53::new(self.buf_data()).fmt(f)?;
            }
            Tag::DVB(TagDVB::Content) => {
                DescDVB0x54::new(self.buf_data()).fmt(f)?;
            }
            Tag::DVB(TagDVB::Teletext) => {
                DescDVB0x56::new(self.buf_data()).fmt(f)?;
            }
            Tag::DVB(TagDVB::AC3) => {
                DescDVB0x6A::new(self.buf_data()).fmt(f)?;
            }
            _ => {
                write!(f, ":data {}", self.data_as_unicode())?;
            }
        }

        Ok(())
    }
}
