use chrono::{DateTime, Local};
use compact_str::CompactString;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StockAccount {
    pub id: CompactString,
    pub owner: CompactString,
    pub expire_on: Option<DateTime<Local>>,
    pub entitlement_id: Option<CompactString>,
    pub account_index: Option<CompactString>,
    pub machine_hash: Option<CompactString>,
}
