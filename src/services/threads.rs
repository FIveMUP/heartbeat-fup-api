use crate::{config::Database, repositories::StockAccountRepository, services::HeartbeatService};
use dashmap::DashMap;
use futures::{stream, StreamExt};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::task::JoinHandle;
use tracing::info;

const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(360);

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

    pub fn spawn_thread(&self, key: &str, server_id: &str, sv_license_key_token: &str) {
        if !self.get(key) {
            let handle = self.tokio_thread(
                Arc::from(key),
                Arc::from(server_id),
                Arc::from(sv_license_key_token),
            );
            self.threads.insert(key.to_owned(), Arc::new(handle));
        }
    }

    #[inline(always)]
    fn tokio_thread(
        &self,
        key: Arc<str>,
        server_id: Arc<str>,
        sv_license_key_token: Arc<str>,
    ) -> JoinHandle<()> {
        let db = self.db.clone();
        let heartbeat = self.heartbeat.clone();
        let stock_repo = StockAccountRepository::new(&db);
        let threads = self.threads.clone();
        let heartbeat_service = Arc::new(HeartbeatService::new());

        self.heartbeat
            .insert(key.clone().to_string(), Instant::now());

        info!("Spawning thread {}", key);

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(10)).await;
            let mut assigned_ids: Vec<String> = vec![];

            loop {
                let now = Instant::now();
                let last_heartbeat = heartbeat.get(&*key).unwrap();

                let new_players = stock_repo.find_all_by_server(&server_id).await;
                let mut new_ids = Vec::new();
                let mut new_assigned_players = Vec::new();

                for player in &new_players {
                    if let Some(id) = &player.id {
                        new_ids.push(id.clone());

                        if !assigned_ids.contains(id) {
                            new_assigned_players.push(player.clone());
                        }
                    }
                }

                if !new_assigned_players.is_empty() {
                    info!(
                        "New players assigned to thread {}: {:?}",
                        key, new_assigned_players
                    );
                } else {
                    info!("No new players assigned to thread {}", key);
                }

                assigned_ids = new_ids;

                let new_assigned_players_stream = stream::iter(new_assigned_players);

                new_assigned_players_stream
                    .for_each_concurrent(None, |mut player| {
                        let key = key.clone();
                        let machine_hash = player.machineHash.take();
                        let entitlement_id = player.entitlementId.take();
                        let sv_license_key_token = sv_license_key_token.clone();
                        let heartbeat_service = heartbeat_service.clone();

                        async move {
                            let result = heartbeat_service
                                .send_ticket(
                                    &machine_hash.unwrap(),
                                    &entitlement_id.unwrap(),
                                    &sv_license_key_token,
                                )
                                .await;

                            match result {
                                Ok(success) => {
                                    info!("Thread {} heartbeat success: {}", key, success);
                                }
                                Err(error) => {
                                    info!("Thread {} heartbeat error: {:?}", key, error);
                                }
                            }
                        }
                    })
                    .await;

                if now.duration_since(*last_heartbeat).gt(&HEARTBEAT_TIMEOUT) {
                    info!("Thread {} is dead", key);
                    threads.remove(&*key);

                    break;
                }

                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        })
    }
}
