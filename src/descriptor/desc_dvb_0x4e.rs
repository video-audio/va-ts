use std::fmt;

// TODO: implement

/// ETSI EN 300 468 V1.15.1
///
/// Extended event descriptor
#[derive(Clone)]
pub struct DescDVB0x4E<'buf> {
    buf: &'buf [u8],
}

impl<'buf> DescDVB0x4E<'buf> {
    #[allow(dead_code)]
    const HEADER_SZ: usize = 4;

    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> DescDVB0x4E<'buf> {
        DescDVB0x4E { buf }
    }
}

impl<'buf> fmt::Debug for DescDVB0x4E<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":dvb-0x4e")
    }
}
