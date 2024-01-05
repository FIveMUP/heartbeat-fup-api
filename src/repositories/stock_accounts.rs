use crate::{config::Database, entities::StockAccount, error::AppResult};
use ahash::AHashMap;
use chrono::{DateTime, Local};
use sqlx::Row;

#[derive(Clone)]
pub struct StockAccountRepository {
    db: Database,
}

impl StockAccountRepository {
    pub fn new(db: &Database) -> Self {
        Self { db: db.clone() }
    }

    pub async fn find_all_by_server(
        &self,
        server: &str,
    ) -> AppResult<AHashMap<String, StockAccount>> {
        let rows = sqlx::query(
            r#"
                SELECT id, owner, expireOn, entitlementId, accountIndex, machineHash
                FROM stock_accounts
                WHERE assignedServer = ?
            "#,
        )
        .bind(server)
        .fetch_all(&self.db.pool)
        .await?;

        let mut map = AHashMap::with_capacity(rows.len());
        for row in rows {
            let id = row.try_get::<String, _>("id")?;

            let account = StockAccount {
                id: row.try_get::<String, _>("id")?.into(),
                owner: row.try_get::<String, _>("owner")?.into(),
                expire_on: row.try_get::<Option<DateTime<Local>>, _>("expireOn")?,
                entitlement_id: row.try_get::<Option<String>, _>("entitlementId")?,
                account_index: row.try_get::<Option<String>, _>("accountIndex")?,
                machine_hash: row.try_get::<Option<String>, _>("machineHash")?,
            };

            map.insert(id, account);
        }

        Ok(map)
    }
}
