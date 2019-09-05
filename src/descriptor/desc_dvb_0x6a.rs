use std::fmt;

// TODO: implement

/// ETSI EN 300 468 V1.15.1
///
/// AC-3 descriptor
#[derive(Clone)]
pub struct DescDVB0x6A<'buf> {
    buf: &'buf [u8],
}

impl<'buf> DescDVB0x6A<'buf> {
    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> DescDVB0x6A<'buf> {
        DescDVB0x6A { buf }
    }
}

impl<'buf> fmt::Debug for DescDVB0x6A<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":dvb-0x6a")
    }
}
