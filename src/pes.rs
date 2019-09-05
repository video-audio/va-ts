use std::fmt;
use std::time::Duration;

use crate::duration_fmt::DurationFmt;
use crate::error::{Error, Kind as ErrorKind};
use crate::rational;
use crate::rational::Rational;
use crate::result::Result;

/// ISO/IEC 13818-1
pub struct Timestamp<'buf> {
    buf: &'buf [u8],
}

impl<'buf> Timestamp<'buf> {
    const SZ: usize = 5;
    const TB: Rational = rational::TB_90KHZ;

    #[inline(always)]
    fn new(buf: &'buf [u8]) -> Timestamp<'buf> {
        Timestamp { buf }
    }

    /// 90kHz
    pub fn value(&self) -> u64 {
        ((u64::from(self.buf[0]) & 0b0000_1110) << 29) // (>> 1 (<< 30))
            | (u64::from(self.buf[1]) << 22)
            | (u64::from(self.buf[2] & 0b1111_1110) << 14) // (>> 1 (<< 15))
            | (u64::from(self.buf[3]) << 7)
            | u64::from((self.buf[4] & 0b1111_1110) >> 1)
    }

    /// nanoseconds
    pub fn ns(&self) -> u64 {
        rational::rescale(self.value(), Self::TB, rational::TB_1NS)
    }
}

impl<'buf> From<&Timestamp<'buf>> for Duration {
    fn from(t: &Timestamp) -> Self {
        Duration::from_nanos(t.ns())
    }
}

impl<'buf> From<Timestamp<'buf>> for Duration {
    fn from(t: Timestamp) -> Self {
        Duration::from(&t)
    }
}

impl<'buf> From<&Timestamp<'buf>> for DurationFmt {
    fn from(t: &Timestamp) -> Self {
        DurationFmt::from_nanos(t.ns())
    }
}

impl<'buf> fmt::Debug for Timestamp<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ":v(90kHz) {} :v(ns) {} :duration {}",
            self.value(),
            self.ns(),
            DurationFmt::from(self)
        )
    }
}

impl<'buf> fmt::Display for Timestamp<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DurationFmt::from(self).fmt(f)
    }
}

/// ISO/IEC 13818-1
#[derive(Debug, PartialEq)]
pub enum StreamID {
    ProgramStreamMap,
    PrivateStream1,
    PaddingStream,
    PrivateStream2,
    AudioStreamNumber(u8),
    VideoStreamNumber(u8),
    ECMStream,
    EMMStream,
    DSMCCStream,
    ISOIEC13522Stream,
    TypeA,
    TypeB,
    TypeC,
    TypeD,
    TypeE,
    AncillaryStream,
    SLPacketizedStream,
    FlexMuxStream,
    MetadataStream,
    ExtendedStreamId,
    ReservedDataStream,
    ProgramStreamDirectory,
    Other(u8),
}

impl StreamID {
    /// if (stream_id != program_stream_map
    /// && stream_id != padding_stream
    /// && stream_id != private_stream_2
    /// && stream_id != ECM
    /// && stream_id != EMM
    /// && stream_id != program_stream_directory
    /// && stream_id != DSMCC_stream
    /// && stream_id != ITU-T Rec. H.222.1 type E stream)
    #[inline(always)]
    pub fn is1(self) -> bool {
        self != StreamID::ProgramStreamMap
            && self != StreamID::PaddingStream
            && self != StreamID::PrivateStream2
            && self != StreamID::ECMStream
            && self != StreamID::EMMStream
            && self != StreamID::ProgramStreamDirectory
            && self != StreamID::DSMCCStream
            && self != StreamID::TypeE
    }

    /// if ( stream_id == program_stream_map
    /// || stream_id == private_stream_2
    /// || stream_id == ECM
    /// || stream_id == EMM
    /// || stream_id == program_stream_directory
    /// || stream_id == DSMCC_stream
    /// || stream_id == ITU-T Rec. H.222.1 type E stream)
    #[inline(always)]
    pub fn is2(self) -> bool {
        self == StreamID::ProgramStreamMap
            || self == StreamID::PrivateStream2
            || self == StreamID::ECMStream
            || self == StreamID::EMMStream
            || self == StreamID::ProgramStreamDirectory
            || self == StreamID::DSMCCStream
            || self == StreamID::TypeE
    }

    /// if ( stream_id == padding_stream)
    #[inline(always)]
    pub fn is3(self) -> bool {
        self == StreamID::PaddingStream
    }
}

impl From<u8> for StreamID {
    fn from(d: u8) -> Self {
        match d {
            0b1011_1100 => StreamID::ProgramStreamMap,
            0b1011_1101 => StreamID::PrivateStream1,
            0b1011_1110 => StreamID::PaddingStream,
            0b1011_1111 => StreamID::PrivateStream2,
            0b1100_0000..=0b1101_1111 => StreamID::AudioStreamNumber(d),
            0b1110_0000..=0b1110_1111 => StreamID::VideoStreamNumber(d),
            0b1111_0000 => StreamID::ECMStream,
            0b1111_0001 => StreamID::EMMStream,
            0b1111_0010 => StreamID::DSMCCStream,
            0b1111_0011 => StreamID::ISOIEC13522Stream,
            0b1111_0100 => StreamID::TypeA,
            0b1111_0101 => StreamID::TypeB,
            0b1111_0110 => StreamID::TypeC,
            0b1111_0111 => StreamID::TypeD,
            0b1111_1000 => StreamID::TypeE,
            0b1111_1001 => StreamID::AncillaryStream,
            0b1111_1010 => StreamID::SLPacketizedStream,
            0b1111_1011 => StreamID::FlexMuxStream,
            0b1111_1100 => StreamID::MetadataStream,
            0b1111_1101 => StreamID::ExtendedStreamId,
            0b1111_1110 => StreamID::ReservedDataStream,
            0b1111_1111 => StreamID::ProgramStreamDirectory,
            _ => StreamID::Other(d),
        }
    }
}

impl From<StreamID> for u8 {
    fn from(sid: StreamID) -> u8 {
        match sid {
            StreamID::ProgramStreamMap => 0b1011_1100,
            StreamID::PrivateStream1 => 0b1011_1101,
            StreamID::PaddingStream => 0b1011_1110,
            StreamID::PrivateStream2 => 0b1011_1111,
            StreamID::AudioStreamNumber(d) => d,
            StreamID::VideoStreamNumber(d) => d,
            StreamID::ECMStream => 0b1111_0000,
            StreamID::EMMStream => 0b1111_0001,
            StreamID::DSMCCStream => 0b1111_0010,
            StreamID::ISOIEC13522Stream => 0b1111_0011,
            StreamID::TypeA => 0b1111_0100,
            StreamID::TypeB => 0b1111_0101,
            StreamID::TypeC => 0b1111_0110,
            StreamID::TypeD => 0b1111_0111,
            StreamID::TypeE => 0b1111_1000,
            StreamID::AncillaryStream => 0b1111_1001,
            StreamID::SLPacketizedStream => 0b1111_1010,
            StreamID::FlexMuxStream => 0b1111_1011,
            StreamID::MetadataStream => 0b1111_1100,
            StreamID::ExtendedStreamId => 0b1111_1101,
            StreamID::ReservedDataStream => 0b1111_1110,
            StreamID::ProgramStreamDirectory => 0b1111_1111,
            StreamID::Other(d) => d,
        }
    }
}

/// ISO/IEC 13818-1
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScramblingControl {
    NotScrabled,
    UserDefined(u8),
}

impl From<u8> for ScramblingControl {
    #[inline(always)]
    fn from(d: u8) -> Self {
        match d {
            0b00 => ScramblingControl::NotScrabled,
            _ => ScramblingControl::UserDefined(d),
        }
    }
}

/// ISO/IEC 13818-1
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PtsDtsFlag {
    No,
    Pts,
    PtsDts,
    Forbidden,
}

impl From<u8> for PtsDtsFlag {
    #[inline(always)]
    fn from(d: u8) -> Self {
        match d {
            0b00 => PtsDtsFlag::No,
            0b10 => PtsDtsFlag::Pts,
            0b11 => PtsDtsFlag::PtsDts,

            _ => PtsDtsFlag::Forbidden,
        }
    }
}

/// ISO/IEC 13818-1
///
/// http://dvd.sourceforge.net/dvdinfo/pes-hdr.html
pub struct PES<'buf> {
    buf: &'buf [u8],
}

impl<'buf> PES<'buf> {
    const HEADER_SZ: usize = 6;
    const HEADER_SZ_1: usize = 3;
    const START_CODE: u32 = 0x0000_0001;
    const PTS_OFFSET_LFT: usize = 9;
    const PTS_OFFSET_RGHT: usize = Self::PTS_OFFSET_LFT + Timestamp::SZ;
    const DTS_OFFSET_LFT: usize = Self::PTS_OFFSET_RGHT;
    const DTS_OFFSET_RGHT: usize = Self::DTS_OFFSET_LFT + Timestamp::SZ;

    #[inline(always)]
    pub fn new(buf: &'buf [u8]) -> PES<'buf> {
        PES { buf }
    }

    #[inline(always)]
    pub fn try_new(buf: &'buf [u8]) -> Result<PES<'buf>> {
        let p = PES::new(buf);
        p.validate()?;
        Ok(p)
    }

    #[inline(always)]
    pub fn validate(&self) -> Result<()> {
        let sz1 = || PES::HEADER_SZ + PES::HEADER_SZ_1 + (self.pes_header_data_length() as usize);

        if self.buf.len() < Self::HEADER_SZ {
            Err(Error::new(ErrorKind::Buf(self.buf.len(), Self::HEADER_SZ)))
        } else if self.start_code() != Self::START_CODE {
            Err(Error::new(ErrorKind::PESStartCode(self.start_code())))
        } else if self.stream_id().is1() && self.buf.len() < sz1() {
            Err(Error::new(ErrorKind::Buf(self.buf.len(), sz1())))
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    fn start_code(&self) -> u32 {
        (u32::from(self.buf[0]) << 16) | (u32::from(self.buf[1]) << 8) | u32::from(self.buf[2])
    }

    #[inline(always)]
    fn stream_id(&self) -> StreamID {
        StreamID::from(self.buf[3])
    }

    #[inline(always)]
    #[allow(dead_code)]
    fn packet_length(&self) -> u16 {
        u16::from(self.buf[4]) << 8 | u16::from(self.buf[5])
    }

    #[inline(always)]
    fn pts_dts_flag(&self) -> Option<PtsDtsFlag> {
        if self.stream_id().is1() {
            Some(PtsDtsFlag::from((self.buf[7] & 0b1100_0000) >> 6))
        } else {
            None
        }
    }

    #[inline(always)]
    fn pes_header_data_length(&self) -> usize {
        usize::from(self.buf[8])
    }

    #[inline(always)]
    pub fn pts(&self) -> Option<Timestamp> {
        self.pts_dts_flag().and_then(|flag| match flag {
            PtsDtsFlag::Pts | PtsDtsFlag::PtsDts => Some(Timestamp::new(
                &self.buf[Self::PTS_OFFSET_LFT..Self::PTS_OFFSET_RGHT],
            )),
            _ => None,
        })
    }

    #[inline(always)]
    pub fn dts(&self) -> Option<Timestamp> {
        self.pts_dts_flag().and_then(|flag| match flag {
            PtsDtsFlag::PtsDts => Some(Timestamp::new(
                &self.buf[Self::DTS_OFFSET_LFT..Self::DTS_OFFSET_RGHT],
            )),
            _ => None,
        })
    }

    #[inline(always)]
    pub fn buf_seek_payload(&self) -> &'buf [u8] {
        if self.stream_id().is1() {
            &self.buf[(Self::HEADER_SZ + Self::HEADER_SZ_1)..]
        } else {
            &self.buf[Self::HEADER_SZ..]
        }
    }
}

impl<'buf> fmt::Debug for PES<'buf> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":PES (")?;

        if let Some(pts) = self.pts() {
            write!(f, ":pts {}", pts)?;
        }

        if let Some(dts) = self.dts() {
            write!(f, " :dts {}", dts)?;
        }

        write!(f, ")")
    }
}
