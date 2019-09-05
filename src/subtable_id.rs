use crate::table_id::TableID;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SubtableID {
    /// (table-id, transport-stream-id(ext) [, version-number])
    PAT(TableID, u16, u8),

    /// (table-id, program-number(ext) [, version-number])
    PMT(TableID, u16, u8),

    /// (table-id, transport-stream-id(ext), original-network-id, version-number)
    SDT(TableID, u16, u16, u8),

    /// (table-id, service-id(ext), transport-stream-id, original-network-id, version-number)
    EIT(TableID, u16, u16, u16, u8),
}

pub trait SubtableIDer {
    fn subtable_id(&self) -> SubtableID;
}
