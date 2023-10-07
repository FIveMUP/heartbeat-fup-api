use crate::{config::Database, services::ThreadService};
use std::sync::Arc;

pub type GlobalState = Arc<GlobalStateInner>;

#[derive(Clone)]
pub struct GlobalStateInner {
    pub threads_service: ThreadService,
}

impl GlobalStateInner {
    pub fn new(database: &Arc<Database>) -> Self {
        Self {
            threads_service: ThreadService::new(database),
        }
    }
}
