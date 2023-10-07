use crate::{
    config::Database,
    repositories::{ServerRepository, StockAccountRepository},
    services::ThreadService,
};
use std::sync::Arc;

pub type GlobalState = Arc<GlobalStateInner>;

#[derive(Clone)]
pub struct GlobalStateInner {
    pub stock_account_repository: StockAccountRepository,
    pub server_repository: ServerRepository,
    pub threads_service: ThreadService,
}

impl GlobalStateInner {
    pub fn new(database: &Arc<Database>) -> Self {
        Self {
            stock_account_repository: StockAccountRepository::new(database),
            server_repository: ServerRepository::new(database),
            threads_service: ThreadService::new(database),
        }
    }
}
