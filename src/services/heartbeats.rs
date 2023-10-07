use crate::config::Database;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::info;
#[derive(Clone)]
pub struct HeartbeatService {
    db: Arc<Database>,
}

impl HeartbeatService {
    pub fn new(db: &Arc<Database>) -> Self {
        Self {
            db: db.clone(),
        }
    }

    // pub fn ticket_heartbeat(&self, )
}
