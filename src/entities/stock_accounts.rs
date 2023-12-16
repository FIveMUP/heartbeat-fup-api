use chrono::{DateTime, Local};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, PartialEq, Eq, Hash)]
pub struct StockAccount {
    pub id: Option<String>,
    pub owner: Option<String>,
    pub expireOn: Option<DateTime<Local>>,
    pub entitlementId: Option<String>,
    pub accountIndex: Option<String>,
    pub machineHash: Option<String>,
}
