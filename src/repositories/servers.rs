use tracing::info;

use crate::{config::Database, entities::Server};
use std::sync::Arc;

#[derive(Clone)]
pub struct ServerRepository {
    db: Arc<Database>,
}

impl ServerRepository {
    pub fn new(db: &Arc<Database>) -> Self {
        Self { db: db.clone() }
    }

    pub async fn find_by_license(&self, license: &str) -> Option<Server> {
        info!("Finding server by license: {}", license);
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
