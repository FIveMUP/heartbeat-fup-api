use crate::{config::Database, entities::StockAccount, error::AppResult};
use ahash::AHashMap;
use chrono::{DateTime, Utc};
use compact_str::CompactString;
use sqlx::Row;

#[derive(Clone)]
pub struct StockAccountRepository {
    db: Database,
}

impl StockAccountRepository {
    pub fn new(db: &Database) -> Self {
        Self { db: db.clone() }
    }

    pub async fn get_count(&self, server: &str) -> AppResult<usize> {
        let row = sqlx::query(
            r#"
                SELECT id
                FROM stock_accounts
                WHERE assignedServer = ?;
            "#,
        )
        .bind(server)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(row.len())
    }

    pub async fn find_all_by_server(
        &self,
        server: &str,
    ) -> AppResult<AHashMap<CompactString, StockAccount>> {
        // TODO: Change database schema to avoid checking for nulls
        let rows = sqlx::query(
            r#"
                SELECT id, owner, expireOn, entitlementId, accountIndex, machineHash
                FROM stock_accounts
                WHERE assignedServer = ?
                AND expireOn IS NOT NULL
                AND entitlementId IS NOT NULL
                AND accountIndex IS NOT NULL
                AND machineHash IS NOT NULL;
            "#,
        )
        .bind(server)
        .fetch_all(&self.db.pool)
        .await?;

        let mut map = AHashMap::with_capacity(rows.len());
        for row in rows {
            let id = row.try_get::<String, _>("id")?.into();

            let account = StockAccount {
                id: row.try_get::<String, _>("id")?.into(),
                owner: row.try_get::<String, _>("owner")?.into(),
                expire_on: row.try_get::<DateTime<Utc>, _>("expireOn")?,
                entitlement_id: row.try_get::<String, _>("entitlementId")?.into(),
                account_index: row.try_get::<String, _>("accountIndex")?.into(),
                machine_hash: row.try_get::<String, _>("machineHash")?.into(),
            };

            map.insert(id, account);
        }

        Ok(map)
    }
}
