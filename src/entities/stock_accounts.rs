use chrono::{DateTime, Local};
use serde::Deserialize;
use sqlx::FromRow;

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct StockAccount {
    #[serde(skip_deserializing)]
    pub id: Option<String>,
    #[serde(skip_deserializing)]
    pub owner: Option<String>,
    #[serde(rename = "expireOn", skip_deserializing)]
    pub expire_on: Option<DateTime<Local>>,
    #[serde(rename = "entitlementId")]
    pub entitlement_id: Option<String>,
    #[serde(rename = "machineHash")]
    pub machine_hash: Option<String>,
}
