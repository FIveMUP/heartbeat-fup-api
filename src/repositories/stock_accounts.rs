use crate::{config::Database, entities::StockAccount};
use std::sync::Arc;

pub struct StockAccountRepository {
    db: Arc<Database>,
}

impl StockAccountRepository {
    pub fn new(db: &Arc<Database>) -> Self {
        Self { db: db.to_owned() }
    }

    pub async fn find_all_by_server(&self, server: &str) -> Vec<StockAccount> {
        sqlx::query_as::<_, StockAccount>(
            r#"
                SELECT id, owner, expireOn, entitlementId, accountIndex, machineHash FROM stock_accounts WHERE assignedServer = ?
            "#
        )
        .bind(server)
        .fetch_all(&self.db.pool)
        .await.unwrap()
    }
}
