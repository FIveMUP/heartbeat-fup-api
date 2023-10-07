use sqlx::FromRow;
use chrono::{DateTime, Local};

#[derive(Debug, FromRow)]
pub struct StockAccount {
    pub id: Option<String>,
    pub owner: Option<String>,
    pub expireOn: Option<DateTime<Local>>,
    // pub assignedServer: Option<String>,
    // pub expireOn: Option,
    pub entitlementId: Option<String>,
    pub machineHash: Option<String>,
}
