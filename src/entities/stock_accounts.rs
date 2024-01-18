use chrono::{DateTime, Utc};
use compact_str::CompactString;

#[derive(Debug, Clone)]
pub struct StockAccount {
    pub id: CompactString,
    pub owner: CompactString,
    pub expire_on: DateTime<Utc>,
    pub entitlement_id: CompactString,
    pub account_index: CompactString,
    pub machine_hash: CompactString,
}
