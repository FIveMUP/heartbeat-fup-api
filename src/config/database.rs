use dotenvy::var;
use sqlx::MySqlPool;
use tracing::info;

#[derive(Debug)]
pub struct Database {
    pub pool: MySqlPool,
}

impl Database {
    pub async fn new() -> Self {
        let pool = MySqlPool::connect(&var("DATABASE_URL").unwrap())
            .await
            .unwrap();

        info!("Connected to database");

        Self { pool }
    }
}
