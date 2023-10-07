use crate::config::Database;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServerRepository {
    db: Arc<Database>,
}

impl ServerRepository {
    pub fn new(db: &Arc<Database>) -> Self {
        Self { db: db.clone() }
    }

    pub async fn find_by_license(&self, license: &str) -> Option<(String,)> {
        sqlx::query_as::<_, (String,)>(
            r#"
                SELECT cfxLicense FROM servers WHERE cfxLicense = ?
            "#,
        )
        .bind(license)
        .fetch_optional(&self.db.pool)
        .await
        .unwrap()
    }
}
