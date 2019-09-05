use std::fmt;

use crate::error::{Error, Kind as ErrorKind};
use crate::iso_639::ISO639;
use crate::result::Result;
use crate::section::{Cursor, Szer, TryNewer};

/// ISO/IEC 13818-1
///
/// ISO 639 language descriptor
#[derive(Clone)]
pub struct Desc0x0A<'buf> {
    buf: &'buf [u8],
}

impl<'buf> Desc0x0A<'buf> {
    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> Desc0x0A<'buf> {
        Desc0x0A { buf }
    }

    #[inline(always)]
    pub fn languages(&self) -> Cursor<'buf, Language> {
        Cursor::new(self.buf)
    }
}

impl<'buf> fmt::Debug for Desc0x0A<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":0x0a :languages")?;

        for resl in self.languages() {
            write!(f, "\n    ")?;
            match resl {
                Ok(l) => {
                    l.fmt(f)?;
                }
                Err(err) => {
                    write!(f, "error parse 0x0a language: {}", err)?;
                }
            }
        }

        Ok(())
    }
}

pub struct Language<'buf> {
    buf: &'buf [u8],
}

impl<'buf> Language<'buf> {
    const SZ: usize = 4;

    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> Language<'buf> {
        Language { buf }
    }

    #[inline(always)]
    pub fn validate(&self) -> Result<()> {
        if self.buf.len() < Self::SZ {
            Err(Error::new(ErrorKind::Buf(self.buf.len(), Self::SZ)))
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    pub fn iso_639_language_code(&self) -> ISO639 {
        ISO639::must_from_bytes_3(self.buf)
    }

    #[inline(always)]
    pub fn audio_type(&self) -> u8 {
        self.buf[3]
    }
}

impl<'buf> Szer for Language<'buf> {
    #[inline(always)]
    fn sz(&self) -> usize {
        Self::SZ
    }
}

impl<'buf> TryNewer<'buf> for Language<'buf> {
    #[inline(always)]
    fn try_new(buf: &'buf [u8]) -> Result<Language<'buf>> {
        let s = Language::new(buf);
        s.validate()?;
        Ok(s)
    }
}

impl<'buf> fmt::Debug for Language<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"        :language (:iso-639 "{}" :audio_type {}/0x{:02X})"#,
            self.iso_639_language_code(),
            self.audio_type(),
            self.audio_type()
        )
    }
}
