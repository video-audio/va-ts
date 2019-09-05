use std::fmt;

use crate::annex_a2::AnnexA2;

// TODO: implement

/// ETSI EN 300 468 V1.15.1
///
/// Service descriptor
#[derive(Clone)]
pub struct DescDVB0x48<'buf> {
    buf: &'buf [u8],
}

impl<'buf> DescDVB0x48<'buf> {
    const HEADER_SZ: usize = 2;

    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> DescDVB0x48<'buf> {
        DescDVB0x48 { buf }
    }

    #[inline(always)]
    pub fn service_type(&self) -> u8 {
        self.buf[0]
    }

    #[inline(always)]
    fn buf_pos_service_provider_name(&self) -> usize {
        Self::HEADER_SZ
    }

    #[inline(always)]
    fn buf_pos_service_name_length(&self) -> usize {
        self.buf_pos_service_provider_name() + (self.service_provider_name_length() as usize)
    }

    #[inline(always)]
    fn buf_pos_service_name(&self) -> usize {
        self.buf_pos_service_name_length() + 1
    }

    #[inline(always)]
    pub fn service_provider_name_length(&self) -> u8 {
        self.buf[1]
    }

    #[inline(always)]
    pub fn service_provider_name(&self) -> &'buf [u8] {
        &self.buf[self.buf_pos_service_provider_name()..self.buf_pos_service_name_length()]
    }

    #[inline(always)]
    pub fn service_name_length(&self) -> u8 {
        self.buf[self.buf_pos_service_name_length()]
    }

    #[inline(always)]
    pub fn service_name(&self) -> &'buf [u8] {
        let lft = self.buf_pos_service_name();
        let rgh = lft + (self.service_name_length() as usize);
        &self.buf[lft..rgh]
    }
}

impl<'buf> fmt::Debug for DescDVB0x48<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ":dvb-0x48 (:service-type 0x{:02}/{}",
            self.service_type(),
            self.service_type()
        )?;

        let mut dst_buf = [0u8; 256];
        let mut dst_str = std::str::from_utf8_mut(&mut dst_buf).unwrap();

        write!(f, " :provider")?;
        match AnnexA2::decode(self.service_provider_name(), &mut dst_str) {
            Ok(..) => write!(f, r#" "{}""#, dst_str),
            Err(err) => write!(f, " (error: {:?})", err),
        }?;

        dst_buf = [0u8; 256];
        dst_str = std::str::from_utf8_mut(&mut dst_buf).unwrap();

        write!(f, " :service")?;
        match AnnexA2::decode(self.service_name(), &mut dst_str) {
            Ok(..) => write!(f, r#" "{}""#, dst_str),
            Err(err) => write!(f, " (error: {:?})", err),
        }?;

        write!(f, ")")
    }
}
