/// ETSI EN 300 468 V1.15.1 (2016-03)
/// ISO/IEC 13818-1
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TableID {
    ProgramAssociationSection,
    ConditionalAccessSection,
    ProgramMapSection,
    TransportStreamDescriptionSection,

    NetworkInformationSectionActualNetwork,
    NetworkInformationSectionOtherNetwork,
    ServiceDescriptionSectionActualTransportStream,

    ServiceDescriptionSectionOtherTransportStream,
    BouquetAssociationSection,
    EISActualTransportStream,
    EISOtherTransportStream,

    EISActualTransportStreamSchedule(u8),
    EISOtherTransportStreamSchedule(u8),

    TimeDateSection,
    RunningStatusSection,
    StuffingSection,
    TimeOffsetSection,
    ApplicationInformationSection,
    ContainerSection,
    RelatedContentSection,
    ContentIdentifierSection,
    MPEFECSection,
    ResolutionNotificationSection,
    MPEIFECSection,
    DiscontinuityInformationSection,
    SelectionInformationSection,

    Reserved(u8),

    Other(u8),
}

impl From<u8> for TableID {
    fn from(d: u8) -> Self {
        match d {
            0x00 => TableID::ProgramAssociationSection,
            0x01 => TableID::ConditionalAccessSection,
            0x02 => TableID::ProgramMapSection,
            0x03 => TableID::TransportStreamDescriptionSection,

            0x40 => TableID::NetworkInformationSectionActualNetwork,
            0x41 => TableID::NetworkInformationSectionOtherNetwork,
            0x42 => TableID::ServiceDescriptionSectionActualTransportStream,

            0x46 => TableID::ServiceDescriptionSectionOtherTransportStream,
            0x4A => TableID::BouquetAssociationSection,
            0x4E => TableID::EISActualTransportStream,
            0x4F => TableID::EISOtherTransportStream,

            0x70 => TableID::TimeDateSection,
            0x71 => TableID::RunningStatusSection,
            0x72 => TableID::StuffingSection,
            0x73 => TableID::TimeOffsetSection,
            0x74 => TableID::ApplicationInformationSection,
            0x75 => TableID::ContainerSection,
            0x76 => TableID::RelatedContentSection,
            0x77 => TableID::ContentIdentifierSection,
            0x78 => TableID::MPEFECSection,
            0x79 => TableID::ResolutionNotificationSection,
            0x7A => TableID::MPEIFECSection,
            0x7E => TableID::DiscontinuityInformationSection,
            0x7F => TableID::SelectionInformationSection,

            0x04..=0x3F | 0x43..=0x45 => TableID::Reserved(d),

            0x50..=0x5F => TableID::EISActualTransportStreamSchedule(d),
            0x60..=0x6F => TableID::EISOtherTransportStreamSchedule(d),

            _ => TableID::Other(d),
        }
    }
}

impl From<TableID> for u8 {
    fn from(id: TableID) -> u8 {
        match id {
            TableID::ProgramAssociationSection => 0x00,
            TableID::ConditionalAccessSection => 0x01,
            TableID::ProgramMapSection => 0x02,
            TableID::TransportStreamDescriptionSection => 0x03,

            TableID::NetworkInformationSectionActualNetwork => 0x40,
            TableID::NetworkInformationSectionOtherNetwork => 0x41,
            TableID::ServiceDescriptionSectionActualTransportStream => 0x42,

            TableID::ServiceDescriptionSectionOtherTransportStream => 0x46,
            TableID::BouquetAssociationSection => 0x4A,
            TableID::EISActualTransportStream => 0x4E,
            TableID::EISOtherTransportStream => 0x4F,

            TableID::TimeDateSection => 0x70,
            TableID::RunningStatusSection => 0x71,
            TableID::StuffingSection => 0x72,
            TableID::TimeOffsetSection => 0x73,
            TableID::ApplicationInformationSection => 0x74,
            TableID::ContainerSection => 0x75,
            TableID::RelatedContentSection => 0x76,
            TableID::ContentIdentifierSection => 0x77,
            TableID::MPEFECSection => 0x78,
            TableID::ResolutionNotificationSection => 0x79,
            TableID::MPEIFECSection => 0x7A,
            TableID::DiscontinuityInformationSection => 0x7E,
            TableID::SelectionInformationSection => 0x7F,

            TableID::Reserved(d) => d,

            TableID::EISActualTransportStreamSchedule(d) => d,
            TableID::EISOtherTransportStreamSchedule(d) => d,

            TableID::Other(d) => d,
        }
    }
}
