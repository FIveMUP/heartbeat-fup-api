use crate::{config::Database, repositories::StockAccountRepository, services::HeartbeatService};
use dashmap::DashMap;
use futures::{stream, StreamExt};
use std::{
    collections::HashSet,
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

    pub fn spawn_thread(&self, key: &str, server_id: &str, sv_licenseKeyToken: &str) {
        if !self.get(key) {
            let handle = self.tokio_thread(key, server_id, sv_licenseKeyToken);
            self.threads.insert(key.to_owned(), Arc::new(handle));
        }
    }

    #[inline(always)]
    fn tokio_thread(&self, key: &str, server_id: &str, sv_licenseKeyToken: &str) -> JoinHandle<()> {
        let key = key.to_owned();
        let db = self.db.clone();
        let heartbeat = self.heartbeat.clone();
        let threads = self.threads.clone();
        let server_id = server_id.to_owned();
        let stock_repo = StockAccountRepository::new(&db);
        let sv_licenseKeyToken = sv_licenseKeyToken.to_owned();
        let heartbeat_service = HeartbeatService::new(&db);

        self.heartbeat.insert(key.clone(), Instant::now());

        info!("Spawning thread {}", key);

        tokio::spawn(async move {
            let mut assigned_ids: Vec<String> = vec![];

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                let measure_ms = tokio::time::Instant::now();
                let now = Instant::now();
                let last_heartbeat = heartbeat.get(&key).unwrap();

                let new_players = stock_repo.find_all_by_server(&server_id).await;
                let new_ids: Vec<_> = new_players
                    .iter()
                    .filter_map(|player| player.id.clone())
                    .collect();

                let new_assigned_players: Vec<_> = new_players
                    .iter()
                    .filter(|player| {
                        player.id.is_some() && !assigned_ids.contains(&player.id.clone().unwrap())
                    })
                    .cloned()
                    .collect();

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
                    .for_each_concurrent(None, |player| {
                        let machine_hash = player.machineHash.clone();
                        let entitlement_id = player.entitlementId.clone();
                        let sv_license_key_token = sv_licenseKeyToken.clone();
                        let heartbeat_service = heartbeat_service.clone();
                        let key = key.clone();
                        async move {
                            let result = heartbeat_service
                                .send_ticket(
                                    machine_hash.as_ref().unwrap(),
                                    entitlement_id.as_ref().unwrap(),
                                    sv_license_key_token.as_ref(),
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

                    threads.remove(&key);

                    break;
                }

                let elapsed = measure_ms.elapsed().as_millis();
            }
        })
    }
}
