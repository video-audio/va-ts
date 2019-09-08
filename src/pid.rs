#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PID {
    PAT,
    CAT,
    TSDT,

    /// NIT, ST
    NIT,
    /// SDT, BAT, ST
    SDT,
    /// EIT, ST CIT (TS 102 323 \[13\])
    EIT,
    /// RST, ST
    RST,
    /// TDT, TOT, ST
    TDT,
    /// network synchronization
    NetworkSynchronization,
    /// RNT (TS 102 323 \[13\])
    RNT,

    InbandSignalling,
    Measurement,
    DIT,
    SIT,

    NULL,

    /// 0x0003...0x000F
    /// 0x0017...0x001B
    Reserved(u16),

    Other(u16),
}

impl PID {
    #[inline(always)]
    pub fn is_section(self) -> bool {
        match self {
            PID::Other(..) | PID::NULL | PID::Reserved(..) => false,
            _ => true,
        }
    }

    #[inline(always)]
    pub fn is_null(self) -> bool {
        match self {
            PID::NULL => true,
            _ => false,
        }
    }

    #[inline(always)]
    pub fn is_other(self) -> bool {
        match self {
            PID::Other(..) => true,
            _ => false,
        }
    }
}

impl From<u16> for PID {
    fn from(d: u16) -> Self {
        match d {
            0x0000 => PID::PAT,
            0x0001 => PID::CAT,
            0x0002 => PID::TSDT,
            0x0003..=0x000F => PID::Reserved(d),
            0x0010 => PID::NIT,
            0x0011 => PID::SDT,
            0x0012 => PID::EIT,
            0x0013 => PID::RST,
            0x0014 => PID::TDT,
            0x0015 => PID::NetworkSynchronization,
            0x0016 => PID::RNT,
            0x0017..=0x001B => PID::Reserved(d),
            0x001C => PID::InbandSignalling,
            0x001D => PID::Measurement,
            0x001E => PID::DIT,
            0x001F => PID::SIT,

            0x1FFF => PID::NULL,

            _ => PID::Other(d),
        }
    }
}

impl From<PID> for u16 {
    fn from(pid: PID) -> u16 {
        match pid {
            PID::PAT => 0x0000,
            PID::CAT => 0x0001,
            PID::TSDT => 0x0002,
            PID::NIT => 0x0010,
            PID::SDT => 0x0011,
            PID::EIT => 0x0012,
            PID::RST => 0x0013,
            PID::TDT => 0x0014,
            PID::NetworkSynchronization => 0x0015,
            PID::RNT => 0x0016,
            PID::InbandSignalling => 0x001C,
            PID::Measurement => 0x001D,
            PID::DIT => 0x001E,
            PID::SIT => 0x001F,

            PID::NULL => 0x1FFF,

            PID::Reserved(d) => d,

            PID::Other(d) => d,
        }
    }
}
