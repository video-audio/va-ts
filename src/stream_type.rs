/// ETSI EN 300 468 V1.15.1 (2016-03)
/// ISO/IEC 13818-1
#[derive(Clone, Debug)]
pub enum StreamType {
    MPEG1Video,
    H262,
    MPEG1Audio, // mp2
    MPEG2Audio, // mp2
    MPEG2TabledData,
    MPEG2PacketizedData, // mp3
    MHEG,
    DSMCCInAPacketizedStream,
    H222AuxiliaryData,
    DSMCCMultiprotocolEncapsulation,
    DSMCCUNMessages,
    DSMCCStreamDescriptors,
    DSMCCTabledData,
    ISOIEC138181AuxiliaryData,
    AAC, // AAC
    MPEG4H263Video,
    MPEG4LOAS,
    MPEG4FlexMux,
    MPEG4FlexMuxTables,
    DSMCCSynchronizedDownloadProtocol,
    PacketizedMetadata,
    SectionedMetadata,
    DSMCCDataCarouselMetadata,
    DSMCCObjectCarouselMetadata,
    SynchronizedDownloadProtocolMetadata,
    IPMP,
    H264,
    MPEG4RawAudio,
    MPEG4Text,
    MPEG4AuxiliaryVideo,
    SVC,
    MVC,
    JPEG2000Video,

    H265,

    ChineseVideoStandard,

    IPMPDRM,
    H262DES64CBC,
    AC3,          // AC3
    SCTESubtitle, // SCTE
    DolbyTrueHDAudio,
    AC3DolbyDigitalPlus,
    DTS8,
    SCTE35,
    AC3DolbyDigitalPlus16,

    // 0x00
    // 0x22
    // 0x23
    // 0x25
    // 0x41
    // 0x43...0x7E
    // 0x88...0x8F
    Reserved(u8),

    Other(u8),
}

impl From<u8> for StreamType {
    fn from(d: u8) -> Self {
        match d {
            0x01 => StreamType::MPEG1Video,
            0x02 => StreamType::H262,
            0x03 => StreamType::MPEG1Audio, // mp2
            0x04 => StreamType::MPEG2Audio, // mp2
            0x05 => StreamType::MPEG2TabledData,
            0x06 => StreamType::MPEG2PacketizedData, // mp3
            0x07 => StreamType::MHEG,
            0x08 => StreamType::DSMCCInAPacketizedStream,
            0x09 => StreamType::H222AuxiliaryData,
            0x0A => StreamType::DSMCCMultiprotocolEncapsulation,
            0x0B => StreamType::DSMCCUNMessages,
            0x0C => StreamType::DSMCCStreamDescriptors,
            0x0D => StreamType::DSMCCTabledData,
            0x0E => StreamType::ISOIEC138181AuxiliaryData,
            0x0F => StreamType::AAC, // AAC
            0x10 => StreamType::MPEG4H263Video,
            0x11 => StreamType::MPEG4LOAS,
            0x12 => StreamType::MPEG4FlexMux,
            0x13 => StreamType::MPEG4FlexMuxTables,
            0x14 => StreamType::DSMCCSynchronizedDownloadProtocol,
            0x15 => StreamType::PacketizedMetadata,
            0x16 => StreamType::SectionedMetadata,
            0x17 => StreamType::DSMCCDataCarouselMetadata,
            0x18 => StreamType::DSMCCObjectCarouselMetadata,
            0x19 => StreamType::SynchronizedDownloadProtocolMetadata,
            0x1A => StreamType::IPMP,
            0x1B => StreamType::H264,
            0x1C => StreamType::MPEG4RawAudio,
            0x1D => StreamType::MPEG4Text,
            0x1E => StreamType::MPEG4AuxiliaryVideo,
            0x1F => StreamType::SVC,
            0x20 => StreamType::MVC,
            0x21 => StreamType::JPEG2000Video,

            0x24 => StreamType::H265,

            0x42 => StreamType::ChineseVideoStandard,

            0x7F => StreamType::IPMPDRM,
            0x80 => StreamType::H262DES64CBC,
            0x81 => StreamType::AC3,          // AC3
            0x82 => StreamType::SCTESubtitle, // SCTE
            0x83 => StreamType::DolbyTrueHDAudio,
            0x84 => StreamType::AC3DolbyDigitalPlus,
            0x85 => StreamType::DTS8,
            0x86 => StreamType::SCTE35,
            0x87 => StreamType::AC3DolbyDigitalPlus16,

            0x00 | 0x22 | 0x23 | 0x25 | 0x41 | 0x43..=0x7E | 0x88..=0x8F => StreamType::Reserved(d),

            _ => StreamType::Other(d),
        }
    }
}

impl From<StreamType> for u8 {
    fn from(st: StreamType) -> u8 {
        match st {
            StreamType::MPEG1Video => 0x01,
            StreamType::H262 => 0x02,
            StreamType::MPEG1Audio => 0x03,
            StreamType::MPEG2Audio => 0x04,
            StreamType::MPEG2TabledData => 0x05,
            StreamType::MPEG2PacketizedData => 0x06,
            StreamType::MHEG => 0x07,
            StreamType::DSMCCInAPacketizedStream => 0x08,
            StreamType::H222AuxiliaryData => 0x09,
            StreamType::DSMCCMultiprotocolEncapsulation => 0x0A,
            StreamType::DSMCCUNMessages => 0x0B,
            StreamType::DSMCCStreamDescriptors => 0x0C,
            StreamType::DSMCCTabledData => 0x0D,
            StreamType::ISOIEC138181AuxiliaryData => 0x0E,
            StreamType::AAC => 0x0F,
            StreamType::MPEG4H263Video => 0x10,
            StreamType::MPEG4LOAS => 0x11,
            StreamType::MPEG4FlexMux => 0x12,
            StreamType::MPEG4FlexMuxTables => 0x13,
            StreamType::DSMCCSynchronizedDownloadProtocol => 0x14,
            StreamType::PacketizedMetadata => 0x15,
            StreamType::SectionedMetadata => 0x16,
            StreamType::DSMCCDataCarouselMetadata => 0x17,
            StreamType::DSMCCObjectCarouselMetadata => 0x18,
            StreamType::SynchronizedDownloadProtocolMetadata => 0x19,
            StreamType::IPMP => 0x1A,
            StreamType::H264 => 0x1B,
            StreamType::MPEG4RawAudio => 0x1C,
            StreamType::MPEG4Text => 0x1D,
            StreamType::MPEG4AuxiliaryVideo => 0x1E,
            StreamType::SVC => 0x1F,
            StreamType::MVC => 0x20,
            StreamType::JPEG2000Video => 0x21,

            StreamType::H265 => 0x24,

            StreamType::ChineseVideoStandard => 0x42,

            StreamType::IPMPDRM => 0x7F,
            StreamType::H262DES64CBC => 0x80,
            StreamType::AC3 => 0x81,
            StreamType::SCTESubtitle => 0x82,
            StreamType::DolbyTrueHDAudio => 0x83,
            StreamType::AC3DolbyDigitalPlus => 0x84,
            StreamType::DTS8 => 0x85,
            StreamType::SCTE35 => 0x86,
            StreamType::AC3DolbyDigitalPlus16 => 0x87,

            StreamType::Reserved(d) => d,

            StreamType::Other(d) => d,
        }
    }
}
