use crate::{config::Database, entities::Server, error::AppResult};
use compact_str::CompactString;
use sqlx::Row;

#[derive(Clone)]
pub struct ServerRepository {
    db: Database,
}

impl ServerRepository {
    pub fn new(db: &Database) -> Self {
        Self { db: db.clone() }
    }

    pub async fn find_by_license(&self, license: &str) -> AppResult<Option<Server>> {
        // TODO: Change database schema to avoid checking for nulls
        let row = sqlx::query(
            r#"
                SELECT * FROM servers
                WHERE cfxLicense = ?
                AND name IS NOT NULL
                AND svLicenseKeyToken IS NOT NULL;
            "#,
        )
        .bind(license)
        .fetch_optional(&self.db.pool)
        .await?;

        if let Some(row) = row {
            let server = Server {
                id: row.try_get::<String, _>("id")?.into(),
                name: row.try_get::<String, _>("name")?.into(),
                cfx_license: row.try_get::<String, _>("cfxLicense")?.into(),
                cfx_code: row
                    .try_get::<Option<String>, _>("cfxCode")?
                    .map(CompactString::from),
                sv_license_key_token: row.try_get::<String, _>("sv_licenseKeyToken")?.into(),
            };

            return Ok(Some(server));
        }

        Ok(None)
    }
}
