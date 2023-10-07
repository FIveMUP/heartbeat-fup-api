use crate::{config::Database, repositories::StockAccountRepository};
use dashmap::DashMap;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::task::JoinHandle;
use tracing::info;

const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone)]
pub struct ThreadService {
    db: Arc<Database>,
    threads: Arc<DashMap<String, Arc<JoinHandle<()>>>>,
    heartbeat: Arc<DashMap<String, Instant>>,
}

impl ThreadService {
    pub fn new(db: &Arc<Database>) -> Self {
        Self {
            db: db.clone(),
            threads: Arc::new(DashMap::new()),
            heartbeat: Arc::new(DashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> bool {
        self.threads.get(key).is_some()
    }

    pub fn heartbeat(&self, key: &str) {
        if let Some(mut heartbeat) = self.heartbeat.get_mut(key) {
            *heartbeat = Instant::now();
        }
    }

    pub fn spawn_thread(&self, key: &str) {
        if !self.get(key) {
            let handle = self.tokio_thread(key);
            self.threads.insert(key.to_owned(), Arc::new(handle));
        }
    }

    #[inline(always)]
    fn tokio_thread(&self, key: &str) -> JoinHandle<()> {
        let key = key.to_owned();
        let db = self.db.clone();
        let heartbeat = self.heartbeat.clone();
        let stock_repo = StockAccountRepository::new(&db);
        let threads = self.threads.clone();

        self.heartbeat.insert(key.clone(), Instant::now());

        info!("Spawning thread {}", key);

        tokio::spawn(async move {
            let stock_accounts = stock_repo.find_all_by_server(&key).await;

            info!("{} stock accounts found", stock_accounts.len());

            /*

               assignedPlayers: [
               {
                   id: "okki_3a",
                   name: "juan",

               },
               ]

               Comparar Stock accounts devuelve los que no estaban añadidos


            */
            // Select * stock_accounts where assignedServer -> ["pepe"] -> Borrar juan de la assignedPlayers
            // new_players = los jugadores nuevos
            // stock_accounts -> assignedPlayers

            // loop {
            //     tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            //     let now = Instant::now();
            //     let last_heartbeat = heartbeat.get(&key).unwrap();

            //     if now.duration_since(*last_heartbeat).gt(&HEARTBEAT_TIMEOUT) {
            //         info!("Thread {} is dead", key);
            //         // Thread is dead

            //         threads.remove(&key);

            //         break;
            //     }

            //     //
            // }
        })
    }
}
