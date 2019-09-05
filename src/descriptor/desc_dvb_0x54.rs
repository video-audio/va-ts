use std::fmt;

// TODO: implement

/// ETSI EN 300 468 V1.15.1
///
/// Content descriptor
#[derive(Clone)]
pub struct DescDVB0x54<'buf> {
    buf: &'buf [u8],
}

impl<'buf> DescDVB0x54<'buf> {
    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> DescDVB0x54<'buf> {
        DescDVB0x54 { buf }
    }
}

impl<'buf> fmt::Debug for DescDVB0x54<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":dvb-0x54")
    }
}
