use crate::error::{Error, Kind as ErrorKind};
use crate::result::Result;
use chrono::prelude::*;
use std::time::Duration;

/// simple binary-coded decimal converter
#[inline(always)]
fn bcd(hex: u8) -> u8 {
    let digit1 = (hex & 0xF0) >> 4;
    let digit2 = hex & 0x0F;

    digit1 * 10 + digit2
}

/// Modified Julian Date to YMD
fn mjb_to_ymd(mjd: u16) -> (i32, u32, u32) {
    let y_y = ((f32::from(mjd) - 15_078.2) / 365.25) as i32;
    let m_m =
        ((f32::from(mjd) - 14_956.1 - ((((y_y as f32) * 365.25) as i32) as f32)) / 30.6001) as i32;
    let d = (i32::from(mjd)
        - 14_956
        - (((y_y as f32) * 365.25) as i32)
        - (((m_m as f32) * 30.6001) as i32)) as u32;

    let k = if m_m == 14 || m_m == 15 { 1 } else { 0 };

    let y = y_y + k + 1900;
    let m = (m_m - 1 - k * 12) as u32;

    (y, m, d)
}

#[allow(dead_code)]
pub fn from_bytes_into_date_time_utc(buf: &[u8]) -> Result<DateTime<Utc>> {
    if buf.len() < 5 {
        return Err(Error::new(ErrorKind::AnnexCBuf(buf.len(), 5)));
    }

    let mjd = (u16::from(buf[0]) << 8) | u16::from(buf[1]);
    let (hh, mm, ss) = (
        u32::from(bcd(buf[2])),
        u32::from(bcd(buf[3])),
        u32::from(bcd(buf[4])),
    );

    let (y, m, d) = mjb_to_ymd(mjd);

    Ok(Utc.ymd(y, m, d).and_hms(hh, mm, ss))
}

#[allow(dead_code)]
pub fn from_bytes_into_duration(buf: &[u8]) -> Result<Duration> {
    if buf.len() < 3 {
        return Err(Error::new(ErrorKind::AnnexCBuf(buf.len(), 3)));
    }

    let (hh, mm, ss) = (
        u64::from(bcd(buf[0])),
        u64::from(bcd(buf[1])),
        u64::from(bcd(buf[2])),
    );

    Ok(Duration::new(hh * 3600 + mm * 60 + ss, 0))
}

#[cfg(test)]
mod tests {
    use super::from_bytes_into_date_time_utc;
    use super::from_bytes_into_duration;
    use crate::error::{Error, Kind as ErrorKind};
    use chrono::prelude::*;
    use std::time::Duration;

    #[test]
    fn parse_datetime() {
        let buf: [u8; 5] = [0xE1, 0x71, 0x15, 0x00, 0x00];

        let dt = from_bytes_into_date_time_utc(&buf);

        assert!(dt.is_ok());
        assert_eq!(
            dt.unwrap_or(Utc::now()),
            Utc.ymd(2016, 11, 21).and_hms(15, 00, 00)
        );
    }

    #[test]
    fn err_parse_datetime() {
        let buf: [u8; 4] = [0xE1, 0x71, 0x15, 0x00];

        let dt = from_bytes_into_date_time_utc(&buf);

        assert!(dt.is_err());
        if let Err(e) = dt {
            assert_eq!(e, Error::new(ErrorKind::AnnexCBuf(4, 5)));
        }
    }

    #[test]
    fn parse_duration() {
        let buf: [u8; 3] = [0x00, 0x45, 0x00];

        let d = from_bytes_into_duration(&buf);

        assert!(d.is_ok());
        assert_eq!(
            d.unwrap_or(Duration::new(0, 0)),
            Duration::from_secs(45 * 60)
        )
    }

    #[test]
    fn err_parse_duration() {
        let buf: [u8; 2] = [0x00, 0x45];

        let d = from_bytes_into_duration(&buf);

        assert!(d.is_err());
        if let Err(e) = d {
            assert_eq!(e, Error::new(ErrorKind::AnnexCBuf(2, 3)));
        }
    }
}
