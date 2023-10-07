use crate::{config::Database, entities::StockAccount};
use std::sync::Arc;

#[derive(Clone)]
pub struct StockAccountRepository {
    db: Arc<Database>,
}

impl StockAccountRepository {
    pub fn new(db: &Arc<Database>) -> Self {
        Self { db: db.clone() }
    }

    pub async fn find_all_by_server(&self, server: &str) -> Vec<StockAccount> {
        sqlx::query_as::<_, StockAccount>(
            r#"
                SELECT id, owner, expireOn, entitlementId, machineHash FROM stock_accounts WHERE assignedServer = ?
            "#
        )
        .bind(server)
        .fetch_all(&self.db.pool)
        .await
        .unwrap()
    }
}
