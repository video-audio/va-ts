use crate::result::Result;
use crate::table_id::TableID;
use std::marker::PhantomData;

pub trait Bufer<'buf> {
    /// borrow a reference to the underlying buffer
    fn buf(&self) -> &'buf [u8];
}

pub const HEADER_SZ: usize = 3;
#[allow(dead_code)]
pub const HEADER_MAX_SECTION_LENGTH: usize = 0x3FD; // 1021

pub(crate) trait WithHeader<'buf>: Bufer<'buf> {
    /// buffer seeked
    #[inline(always)]
    fn b(&self) -> &'buf [u8] {
        self.buf()
    }

    #[inline(always)]
    fn table_id(&self) -> TableID {
        TableID::from(self.b()[0])
    }

    /// if set to 1 (true) - 4th and 5th bytes
    /// are table-id-extension
    ///
    /// must be set to 1 for:
    /// PAT, CAT, PMT
    /// NIT, EIT, BAT, SDT
    #[inline(always)]
    fn section_syntax_indicator(&self) -> bool {
        (self.b()[1] & 0b1000_0000) != 0
    }

    #[inline(always)]
    fn section_length(&self) -> u16 {
        (u16::from(self.b()[1] & 0b0000_1111) << 8) | u16::from(self.b()[2])
    }

    /// complete section length
    #[inline(always)]
    fn sz(&self) -> usize {
        HEADER_SZ + usize::from(self.section_length())
    }
}

pub trait WithTableIDExtension<'buf>: Bufer<'buf> {
    /// buffer seeked
    #[inline(always)]
    fn b(&self) -> &'buf [u8] {
        self.buf()
    }

    #[inline(always)]
    fn table_id_extension(&self) -> u16 {
        (u16::from(self.b()[3]) << 8) | u16::from(self.b()[4])
    }
}

pub const SYNTAX_SECTION_SZ: usize = 5;

pub(crate) trait WithSyntaxSection<'buf>: Bufer<'buf> {
    /// buffer seeked
    #[inline(always)]
    fn b(&self) -> &'buf [u8] {
        &self.buf()[HEADER_SZ..]
    }

    #[inline(always)]
    #[allow(dead_code)]
    fn version_number(&self) -> u8 {
        (self.b()[2] & 0b0011_1110) >> 1
    }

    #[inline(always)]
    fn current_next_indicator(&self) -> bool {
        (self.b()[2] & 0b0000_0001) != 0
    }

    #[inline(always)]
    fn section_number(&self) -> u8 {
        self.b()[3]
    }

    #[inline(always)]
    fn last_section_number(&self) -> u8 {
        self.b()[4]
    }
}

pub trait Szer {
    fn sz(&self) -> usize;
}

pub trait TryNewer<'buf> {
    fn try_new(buf: &'buf [u8]) -> Result<Self>
    where
        Self: Sized;
}

pub struct Cursor<'buf, T> {
    buf: &'buf [u8],
    phantom: PhantomData<T>,
}

impl<'buf, T> Cursor<'buf, T> {
    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> Cursor<'buf, T> {
        Cursor {
            buf,
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    fn buf_drain(&mut self) {
        self.buf = &self.buf[self.buf.len()..];
    }
}

impl<'buf, T> Iterator for Cursor<'buf, T>
where
    T: TryNewer<'buf> + Szer,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.is_empty() {
            return None;
        }

        let row = match T::try_new(self.buf) {
            Ok(row) => row,
            Err(e) => {
                self.buf_drain();
                return Some(Err(e));
            }
        };

        // seek buf
        if self.buf.len() > row.sz() {
            self.buf = &self.buf[row.sz()..];
        } else {
            self.buf_drain();
        }

        Some(Ok(row))
    }
}

pub const CRC32_SZ: usize = 4;

pub(crate) trait WithCRC32<'buf>: Bufer<'buf> {}
