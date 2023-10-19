use crate::{config::Database, entities::Server};
use std::sync::Arc;

pub struct ServerRepository {
    db: Arc<Database>,
}

impl ServerRepository {
    pub fn new(db: &Arc<Database>) -> Self {
        Self { db: db.to_owned() }
    }

    pub async fn find_by_license(&self, license: &str) -> Option<Server> {
        sqlx::query_as::<_, Server>(
            r#"
                SELECT * FROM servers WHERE cfxLicense = ?
            "#,
        )
        .bind(license)
        .fetch_optional(&self.db.pool)
        .await
        .unwrap()
    }
}
