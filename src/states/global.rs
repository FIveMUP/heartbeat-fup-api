use crate::{
    config::Database,
    repositories::{ServerRepository, StockAccountRepository},
    services::{FivemService, ThreadService},
};

#[derive(Clone)]
pub struct GlobalState {
    pub stock_account_repository: StockAccountRepository,
    pub server_repository: ServerRepository,
    pub threads_service: ThreadService,
}

impl GlobalState {
    pub fn new(database: Database, fivem_service: &'static FivemService) -> Self {
        Self {
            stock_account_repository: StockAccountRepository::new(&database),
            server_repository: ServerRepository::new(&database),
            threads_service: ThreadService::new(&database, fivem_service),
        }
    }
}
