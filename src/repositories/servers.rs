use compact_str::CompactString;
use sqlx::Row;

use crate::{config::Database, entities::Server, error::AppResult};

#[derive(Clone)]
pub struct ServerRepository {
    db: Database,
}

impl ServerRepository {
    pub fn new(db: &Database) -> Self {
        Self { db: db.clone() }
    }

    pub async fn find_by_license(&self, license: &str) -> AppResult<Option<Server>> {
        let row = sqlx::query(
            r#"
                SELECT * FROM servers WHERE cfxLicense = ?
            "#,
        )
        .bind(license)
        .fetch_optional(&self.db.pool)
        .await?;

        if let Some(row) = row {
            let server = Server {
                id: row.try_get::<String, _>("id")?.into(),
                name: row
                    .try_get::<Option<String>, _>("name")?
                    .map(CompactString::new),
                cfx_license: row.try_get::<String, _>("cfxLicense")?.into(),
                cfx_code: row
                    .try_get::<Option<String>, _>("cfxCode")?
                    .map(CompactString::new),
                sv_license_key_token: row
                    .try_get::<Option<String>, _>("sv_licenseKeyToken")?
                    .map(CompactString::new),
            };

            return Ok(Some(server));
        }

        Ok(None)
    }
}
