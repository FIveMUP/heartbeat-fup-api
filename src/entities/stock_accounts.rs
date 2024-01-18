use chrono::{DateTime, Local};
use compact_str::CompactString;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StockAccount {
    pub id: CompactString,
    pub owner: CompactString,
    pub expire_on: DateTime<Local>,
    pub entitlement_id: CompactString,
    pub account_index: CompactString,
    pub machine_hash: CompactString,
}
