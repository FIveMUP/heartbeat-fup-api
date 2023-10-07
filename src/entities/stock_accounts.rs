use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct StockAccount {
    pub id: Option<String>,
    pub owner: Option<String>,
    pub expireOn: Option<String>,
    // pub assignedServer: Option<String>,
    // pub expireOn: Option,
    pub entitlementId: Option<String>,
    pub machineHash: Option<String>,
}
