use std::sync::Arc;

use crate::config::Database;

#[derive(Clone)]
pub struct GlobalState {
    pub database: String,
}

impl GlobalState {
    pub fn new(_database: Arc<Database>) -> Self {
        Self {
            database: "fe".to_string(),
        }
    }
}
