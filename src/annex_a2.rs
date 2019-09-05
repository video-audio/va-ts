use crate::error::{Error, Kind as ErrorKind};
use std::convert::TryFrom;

#[derive(Clone, Copy, Debug)]
pub enum TableA3 {
    IsoIec8859_5,
    IsoIec8859_6,
    IsoIec8859_7,
    IsoIec8859_8,
    IsoIec8859_9,
    IsoIec8859_10,
    IsoIec8859_11,
    IsoIec8859_13,
    IsoIec8859_14,
    IsoIec8859_15,

    IsoIec10646,
    KSX10012004,
    Gb2312_1980,
    Big5subsetOfIsoIec10646,
    Utf8encodingOfIsoIec10646,
    DescribedByEncodingTypeId,

    Reserved(u8),
}

impl TableA3 {
    pub fn encoding(self) -> Option<&'static encoding_rs::Encoding> {
        match self {
            TableA3::IsoIec8859_5 => Some(encoding_rs::ISO_8859_5),
            TableA3::IsoIec8859_6 => Some(encoding_rs::ISO_8859_6),
            TableA3::IsoIec8859_7 => Some(encoding_rs::ISO_8859_7),
            TableA3::IsoIec8859_8 => Some(encoding_rs::ISO_8859_8),
            TableA3::IsoIec8859_13 => Some(encoding_rs::ISO_8859_13),
            TableA3::IsoIec8859_14 => Some(encoding_rs::ISO_8859_14),
            TableA3::IsoIec8859_15 => Some(encoding_rs::ISO_8859_15),
            TableA3::Big5subsetOfIsoIec10646 => Some(encoding_rs::BIG5),
            TableA3::Gb2312_1980 => Some(encoding_rs::GBK),
            TableA3::Utf8encodingOfIsoIec10646 => Some(encoding_rs::UTF_8),
            _ => None,
        }
    }
}

impl TryFrom<u8> for TableA3 {
    type Error = Error;

    fn try_from(d: u8) -> Result<Self, self::Error> {
        if d > 0x1F {
            return Err(Error::new(ErrorKind::AnnexA2TableA3Unexpected(d)));
        }

        Ok(match d {
            0x01 => TableA3::IsoIec8859_5,
            0x02 => TableA3::IsoIec8859_6,
            0x03 => TableA3::IsoIec8859_7,
            0x04 => TableA3::IsoIec8859_8,
            0x05 => TableA3::IsoIec8859_9,
            0x06 => TableA3::IsoIec8859_10,
            0x07 => TableA3::IsoIec8859_11,

            0x08 => TableA3::Reserved(d),

            0x09 => TableA3::IsoIec8859_13,
            0x0A => TableA3::IsoIec8859_14,
            0x0B => TableA3::IsoIec8859_15,

            0x0C..=0x0F => TableA3::Reserved(d),

            0x11 => TableA3::IsoIec10646,
            0x12 => TableA3::KSX10012004,
            0x13 => TableA3::Gb2312_1980,
            0x14 => TableA3::Big5subsetOfIsoIec10646,
            0x15 => TableA3::Utf8encodingOfIsoIec10646,

            0x16..=0x1E => TableA3::Reserved(d),

            0x1F => TableA3::DescribedByEncodingTypeId,

            _ => panic!("(annex-a2 table-a3 parse) unexpected value;"),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TableA4 {
    IsoIec8859_1,
    IsoIec8859_2,
    IsoIec8859_3,
    IsoIec8859_4,
    IsoIec8859_5,
    IsoIec8859_6,
    IsoIec8859_7,
    IsoIec8859_8,
    IsoIec8859_9,
    IsoIec8859_10,
    IsoIec8859_11,
    IsoIec8859_13,
    IsoIec8859_14,
    IsoIec8859_15,

    Reserved(u8, u8),
}

impl TableA4 {
    const SYNC_BYTE: u8 = 0x10;
}

impl TableA4 {
    pub fn encoding(self) -> Option<&'static encoding_rs::Encoding> {
        match self {
            TableA4::IsoIec8859_1 => Some(encoding_rs::UTF_8),
            TableA4::IsoIec8859_2 => Some(encoding_rs::ISO_8859_2),
            TableA4::IsoIec8859_3 => Some(encoding_rs::ISO_8859_3),
            TableA4::IsoIec8859_4 => Some(encoding_rs::ISO_8859_4),
            TableA4::IsoIec8859_5 => Some(encoding_rs::ISO_8859_5),
            TableA4::IsoIec8859_6 => Some(encoding_rs::ISO_8859_6),
            TableA4::IsoIec8859_7 => Some(encoding_rs::ISO_8859_7),
            TableA4::IsoIec8859_8 => Some(encoding_rs::ISO_8859_8),
            TableA4::IsoIec8859_10 => Some(encoding_rs::ISO_8859_10),
            TableA4::IsoIec8859_13 => Some(encoding_rs::ISO_8859_13),
            TableA4::IsoIec8859_14 => Some(encoding_rs::ISO_8859_14),
            TableA4::IsoIec8859_15 => Some(encoding_rs::ISO_8859_15),
            _ => None,
        }
    }
}

impl<'buf> TryFrom<&'buf [u8]> for TableA4 {
    type Error = Error;

    fn try_from(buf: &'buf [u8]) -> Result<Self, self::Error> {
        if buf.len() < 3 {
            return Err(Error::new(ErrorKind::AnnexA2TableA4Buf(buf.len(), 3)));
        }

        let (b1, b2, b3) = (buf[0], buf[1], buf[2]);

        if b1 != Self::SYNC_BYTE {
            return Err(Error::new(ErrorKind::AnnexA2TableA4Unexpected(b1)));
        }

        Ok(match (b2, b3) {
            (0x00, 0x01) => TableA4::IsoIec8859_1,
            (0x00, 0x02) => TableA4::IsoIec8859_2,
            (0x00, 0x03) => TableA4::IsoIec8859_3,
            (0x00, 0x04) => TableA4::IsoIec8859_4,
            (0x00, 0x05) => TableA4::IsoIec8859_5,
            (0x00, 0x06) => TableA4::IsoIec8859_6,
            (0x00, 0x07) => TableA4::IsoIec8859_7,
            (0x00, 0x08) => TableA4::IsoIec8859_8,
            (0x00, 0x09) => TableA4::IsoIec8859_9,
            (0x00, 0x0A) => TableA4::IsoIec8859_10,
            (0x00, 0x0B) => TableA4::IsoIec8859_11,
            (0x00, 0x0D) => TableA4::IsoIec8859_13,
            (0x00, 0x0E) => TableA4::IsoIec8859_14,
            (0x00, 0x0F) => TableA4::IsoIec8859_15,

            (0x00, 0x00) => TableA4::Reserved(b1, b2),
            (0x00, 0x0C) => TableA4::Reserved(b1, b2),
            (0x00, 0x10..=0xFF) => TableA4::Reserved(b1, b2),
            (0x01..=0xFF, 0x00..=0xFF) => TableA4::Reserved(b1, b2),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AnnexA2 {
    A3(TableA3),
    A4(TableA4),

    Reserved(u8),
    Zero,

    Default,
}

/// ETSI EN 300 468 V1.15.1
impl AnnexA2 {
    fn encoding(self) -> Option<&'static encoding_rs::Encoding> {
        match self {
            AnnexA2::A3(a3) => a3.encoding(),
            AnnexA2::A4(a4) => a4.encoding(),
            AnnexA2::Default => Some(encoding_rs::UTF_8),
            AnnexA2::Reserved(..) => None,
            AnnexA2::Zero => None,
        }
    }

    // TODO: maybe use "encoding" (rust-encoding) crate?
    pub fn decode<'buf>(src_buf: &'buf [u8], dst_str: &'buf mut str) -> Result<AnnexA2, Error> {
        let a2 = AnnexA2::try_from(src_buf)?;

        let src_buf = &src_buf[a2.sz()..];

        let encoding = match a2.encoding() {
            Some(encoding) => encoding,
            None => return Err(Error::new(ErrorKind::AnnexA2UnsupportedEncoding)),
        };

        let mut decoder = encoding.new_decoder();

        let (result, _, _, had_errors) = decoder.decode_to_str(src_buf, dst_str, false);

        match result {
            encoding_rs::CoderResult::InputEmpty => {
                // We have consumed the current input buffer
            }
            encoding_rs::CoderResult::OutputFull => {}
        }

        if had_errors {
            Err(Error::new(ErrorKind::AnnexA2Decode))
        } else {
            Ok(a2)
        }
    }

    // sz to skip in buffer
    fn sz(self) -> usize {
        match self {
            AnnexA2::A3(..) => 1,
            AnnexA2::A4(..) => 3,
            AnnexA2::Default => 0,
            _ => 0,
        }
    }
}

impl<'buf> TryFrom<&'buf [u8]> for AnnexA2 {
    type Error = Error;

    fn try_from(buf: &'buf [u8]) -> Result<Self, self::Error> {
        if buf.is_empty() {
            return Err(Error::new(ErrorKind::AnnexA2EmptyBuf));
        }

        let v = match buf[0] {
            0x00 => AnnexA2::Zero,

            0x20..=0xFF => AnnexA2::Default,

            0x01..=0x07 | 0x09..=0x0B | 0x11..=0x15 | 0x1F => {
                AnnexA2::A3(TableA3::try_from(buf[0])?)
            }

            0x10 => AnnexA2::A4(TableA4::try_from(buf)?),

            0x08 | 0x0C..=0x0F | 0x16..=0x1E => AnnexA2::Reserved(buf[0]),
        };

        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn decode() {}
}
