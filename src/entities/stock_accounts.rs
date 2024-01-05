use chrono::{DateTime, Local};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StockAccount {
    pub id: String,
    pub owner: String,
    pub expire_on: Option<DateTime<Local>>,
    pub entitlement_id: Option<String>,
    pub account_index: Option<String>,
    pub machine_hash: Option<String>,
}
