use crate::{
    config::Database, entities::StockAccount, repositories::StockAccountRepository,
    services::HeartbeatService,
};
use ahash::AHashMap;
use futures::{stream, StreamExt};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::task::JoinHandle;
use tracing::info;

const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct ThreadService {
    db: Arc<Database>,
    threads: Arc<Mutex<AHashMap<String, Arc<JoinHandle<()>>>>>,
    heartbeat: Arc<Mutex<AHashMap<String, Instant>>>,
}

impl ThreadService {
    pub fn new(db: &Arc<Database>) -> Self {
        Self {
            db: db.clone(),
            threads: Arc::new(Mutex::new(AHashMap::new())),
            heartbeat: Arc::new(Mutex::new(AHashMap::new())),
        }
    }

    pub fn get(&self, key: &str) -> bool {
        self.threads.lock().unwrap().contains_key(key)
    }

    pub fn heartbeat(&self, key: &str) {
        // if let Some(mut heartbeat) = self.heartbeat.get_mut(key) {
        //     *heartbeat = Instant::now();
        // }
        let mut heartbeat_map = self.heartbeat.lock().unwrap();
        if let Some(heartbeat) = heartbeat_map.get_mut(key) {
            info!("Heartbeat received for {}", key);
            *heartbeat = Instant::now();
        }
    }

    pub fn spawn_thread(&self, key: &str, server_id: &str, sv_license_key_token: &str) {
        if !self.get(key) {
            let handle = self.tokio_thread(key, server_id, sv_licenseKeyToken);
            self.threads
                .lock()
                .unwrap()
                .insert(key.to_owned(), Arc::new(handle));
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
        let sv_licenseKeyToken = sv_licenseKeyToken.to_owned();
        let heartbeat_service = HeartbeatService::new();

        self.heartbeat
            .lock()
            .unwrap()
            .insert(key.clone(), Instant::now());

        info!("Spawning thread for {}", key);

        tokio::spawn(async move {
            info!("Spawned thread for {}", key);
            let mut assigned_ids: Vec<_> = vec![];

            loop {
                let last_heartbeat = {
                    let lock_heartbeat = heartbeat.lock().unwrap();
                    *lock_heartbeat.get(&key).unwrap()
                };
                tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;
                let measure_ms = tokio::time::Instant::now();
                let now: Instant = Instant::now();

                let new_players = stock_repo.find_all_by_server(&server_id).await;

                let new_ids: Vec<_> = new_players
                    .iter()
                    .filter_map(|player| player.id.clone())
                    .collect();

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
                        "New players assigned to  {}: {:?}",
                        key,
                        new_assigned_players.len()
                    );
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
                                    machine_hash.as_ref().unwrap_or(&"".to_string()),
                                    entitlement_id.as_ref().unwrap_or(&"".to_string()),
                                    sv_license_key_token.as_ref(),
                                )
                                .await;

                            match result {
                                Ok(_) => {
                                    // info!("Thread {} ticket success: {}", key, success);
                                }
                                Err(error) => {
                                    info!("Thread {} ticket error: {:?}", key, error);
                                }
                            }
                        }
                    })
                    .await;

                let assigned_ids_stream = stream::iter(new_players);

                assigned_ids_stream
                    .for_each_concurrent(None, |player| {
                        let key = key.clone();
                        let machine_hash = player.machineHash.clone();
                        let entitlement_id = player.entitlementId.clone();
                        let heartbeat_service = heartbeat_service.clone();
                        async move {
                            let result = heartbeat_service
                                .send_entitlement(
                                    machine_hash.as_ref().unwrap_or(&"".to_string()),
                                    entitlement_id.as_ref().unwrap_or(&"".to_string()),
                                )
                                .await;
                            match result {
                                Ok(_) => {
                                    // info!("Thread {} heartbeat success: {}", key, success);
                                }
                                Err(error) => {
                                    info!("Thread {} heartbeat error: {:?}", key, error);
                                }
                            }
                        }
                    })
                    .await;

                if now.duration_since(last_heartbeat).gt(&HEARTBEAT_TIMEOUT) {
                    let mut threads_map = threads.lock().unwrap();
                    threads_map.remove(&key);
                    info!("Thread {} timed out, killing", key);
                    break;
                }

                let elapsed = measure_ms.elapsed().as_millis();
                info!("Thread {} took {}ms for {} bots", key, elapsed, assigned_ids.len());
            }
        })
    }
}
