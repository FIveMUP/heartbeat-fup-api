use crate::{
    config::Database,
    entities::StockAccount,
    error::{AppResult, ThreadError},
    repositories::StockAccountRepository,
    services::HeartbeatService,
};
use ahash::AHashMap;
use futures::{stream, StreamExt};
use parking_lot::{Mutex, RwLock};
use std::{sync::Arc, time::Duration};
use tokio::{task::JoinHandle, time::Instant};
use tracing::{error, info, warn};

const THREAD_SLEEP_TIME: Duration = Duration::from_secs(7);
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct ThreadService {
    stock_repo: Arc<StockAccountRepository>,
    heartbeat: Arc<RwLock<AHashMap<String, Mutex<Instant>>>>,
    threads: Arc<RwLock<AHashMap<String, Arc<JoinHandle<()>>>>>,
}

impl ThreadService {
    pub fn new(db: &Database) -> Self {
        Self {
            threads: Arc::new(RwLock::new(AHashMap::new())),
            heartbeat: Arc::new(RwLock::new(AHashMap::new())),
            stock_repo: Arc::new(StockAccountRepository::new(db)),
        }
    }

    pub fn get(&self, key: &str) -> bool {
        self.threads.read().contains_key(key)
    }

    pub fn heartbeat(&self, key: &str) -> AppResult<()> {
        let heartbeat_map = self.heartbeat.read();

        if let Some(heartbeat) = heartbeat_map.get(key) {
            let mut heartbeat = heartbeat.lock();
            *heartbeat = Instant::now();
        } else {
            Err(ThreadError::NotFound)?
        }

        Ok(())
    }

    pub async fn spawn_thread(
        &self,
        key: String,
        server_id: String,
        sv_license_key_token: String,
        server_name: String,
    ) -> AppResult<()> {
        if !self.get(&key) {
            let handle = self
                .tokio_thread(key.clone(), server_id, sv_license_key_token, server_name)
                .await;

            info!("Thread {:?}", handle);
            let mut threads = self.threads.write();
            threads.insert(key.to_owned(), Arc::new(handle));

            Ok(())
        } else {
            Err(ThreadError::AlreadyExists)?
        }
    }

    #[inline(always)]
    async fn tokio_thread(
        &self,
        key: String,
        server_id: String,
        sv_license_key_token: String,
        server_name: String,
    ) -> JoinHandle<()> {
        let stock_repo = self.stock_repo.clone();
        let threads = self.threads.clone();
        let heartbeat = self.heartbeat.clone();
        let heartbeat_service = HeartbeatService::new();

        {
            let mut heartbeat = heartbeat.write();
            heartbeat.insert(key.clone(), Mutex::new(Instant::now()));
        }

        info!("Spawning thread for {}", server_name);

        tokio::spawn(async move {
            info!("Spawned thread for {}", server_name);
            let sv_license_key_token: Arc<str> = Arc::from(sv_license_key_token.to_string());
            let new_players: Arc<RwLock<AHashMap<String, StockAccount>>> =
                Arc::from(RwLock::new(AHashMap::new()));
            let assigned_players: Arc<RwLock<AHashMap<String, StockAccount>>> =
                Arc::from(RwLock::new(AHashMap::new()));

            loop {
                tokio::time::sleep(THREAD_SLEEP_TIME).await;
                let now = tokio::time::Instant::now();

                {
                    let mut heartbeats = heartbeat.upgradable_read();
                    let last_heartbeat = heartbeats.get(&key).unwrap().lock();

                    if now.duration_since(*last_heartbeat).gt(&HEARTBEAT_TIMEOUT) {
                        drop(last_heartbeat);
                        threads.write().remove(&key);

                        heartbeats.with_upgraded(|heartbeats| {
                            heartbeats.remove(&key);
                        });

                        info!("Thread {} timed out", server_name);
                        return;
                    }
                }

                {
                    let Ok(db_players) = stock_repo.find_all_by_server(&server_id).await else {
                        error!("Thread {} db error", server_name);
                        break;
                    };

                    let time = chrono::Utc::now();
                    let mut new_players = new_players.write();
                    let mut assigned_players = assigned_players.write();

                    for (id, player) in db_players.iter() {
                        if let Some(expire) = &player.expire_on {
                            if expire.lt(&time) {
                                info!("Bot {} expired at {}", id, expire);
                                continue;
                            }
                        }

                        if !assigned_players.contains_key(id) {
                            new_players.insert(id.clone(), player.clone());
                            assigned_players.insert(id.clone(), player.clone());
                        }
                    }

                    assigned_players.retain(|id, _player| db_players.contains_key(id));
                }

                // New players
                if let Err(err) = tokio::task::spawn({
                    let heartbeat_service = heartbeat_service.clone();
                    let cloned_new_players = new_players.clone();

                    async move {
                        let new_players = cloned_new_players.read();

                        if new_players.is_empty() {
                            return;
                        }

                        stream::iter(new_players.iter())
                            .for_each_concurrent(None, |(_id, player)| {
                                let heartbeat_service = heartbeat_service.clone();

                                async move {
                                     if player.machine_hash.is_none() || player.entitlement_id.is_none() || player.account_index.is_none() {
                                        warn!("Player {} missing machineHash, entitlementId or accountIndex", &player.id);
                                        return;
                                    }

                                    let result = heartbeat_service
                                        .send_entitlement(
                                            player.machine_hash.as_ref().unwrap(),
                                            player.entitlement_id.as_ref().unwrap(),
                                            player.account_index.as_ref().unwrap(),
                                        )
                                        .await;

                                    if let Err(error) = result {
                                        info!(
                                            "Player {} heartbeat error: {:?}",
                                            &player.id, error
                                        );
                                    }
                                }
                            })
                            .await;
                    }
                }).await {
                    error!("Thread {} new Players error: {:?}", server_name, err);
                };

                // Assigned players
                if let Err(err) = tokio::task::spawn({
                    let sv_license_key_token = sv_license_key_token.clone();
                    let heartbeat_service = heartbeat_service.clone();
                    let assigned_players = assigned_players.clone();

                    async move {
                        let assigned_players = assigned_players.read();

                        if assigned_players.is_empty() {
                            return;
                        }

                        stream::iter(assigned_players.iter())
                            .for_each_concurrent(None, |(_id, player)| {
                                let sv_license_key_token = sv_license_key_token.clone();
                                let heartbeat_service = heartbeat_service.clone();

                                async move {
                                    if player.machine_hash.is_none() || player.entitlement_id.is_none() || player.account_index.is_none() {
                                        warn!("Player {} missing machineHash, entitlementId or accountIndex", &player.id);
                                        return;
                                    }

                                    let result = heartbeat_service
                                        .send_ticket(
                                            player.machine_hash.as_ref().unwrap(),
                                            player.entitlement_id.as_ref().unwrap(),
                                            player.account_index.as_ref().unwrap(),
                                            &sv_license_key_token,
                                        )
                                        .await;

                                    if let Err(error) = result {
                                        info!(
                                            "Player {} ticket error: {:?}",
                                            &player.id, error
                                        );
                                    }
                                }
                            })
                            .await
                    }
                }).await {
                    error!("Thread {} assigned Players error: {:?}", server_name, err);
                };

                let bots = assigned_players.read().len();
                info!(
                    "Thread {:20} took {:5}ms for {:5} bots",
                    server_name,
                    now.elapsed().as_millis(),
                    bots
                );

                new_players.write().clear();
            }
        })
    }
}
