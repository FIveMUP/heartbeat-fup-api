use crate::{
    config::Database,
    error::{AppResult, ThreadError},
    repositories::StockAccountRepository,
    services::HeartbeatService,
};
use ahash::{AHashMap, AHashSet};
use futures::{stream, StreamExt};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, RwLock},
    task::JoinHandle,
    time::Instant,
};
use tracing::info;

const THREAD_SLEEP_TIME: Duration = Duration::from_secs(6);
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct ThreadService {
    stock_repo: Arc<StockAccountRepository>,
    heartbeat: Arc<RwLock<AHashMap<String, Mutex<Instant>>>>,
    threads: Arc<RwLock<AHashMap<String, Arc<JoinHandle<()>>>>>,
}

impl ThreadService {
    pub fn new(db: &Arc<Database>) -> Self {
        Self {
            threads: Arc::new(RwLock::new(AHashMap::new())),
            heartbeat: Arc::new(RwLock::new(AHashMap::new())),
            stock_repo: Arc::new(StockAccountRepository::new(db)),
        }
    }

    pub async fn get(&self, key: &str) -> bool {
        self.threads.read().await.contains_key(key)
    }

    pub async fn heartbeat(&self, key: &str) -> AppResult<()> {
        let heartbeat_map = self.heartbeat.read().await;

        if let Some(heartbeat) = heartbeat_map.get(key) {
            let mut heartbeat = heartbeat.lock().await;
            *heartbeat = Instant::now();
        } else {
            Err(ThreadError::NotFound)?
        }

        Ok(())
    }

    pub async fn spawn_thread(
        &self,
        key: &str,
        server_id: &str,
        sv_license_key_token: &str,
        server_name: &str,
    ) -> AppResult<()> {
        if !self.get(key).await {
            let handle = self
                .tokio_thread(
                    Arc::from(key),
                    Arc::from(server_id),
                    Arc::from(sv_license_key_token),
                    Arc::from(server_name),
                )
                .await;

            info!("Thread {:#?}", handle);
            let mut threads = self.threads.write().await;
            threads.insert(key.to_owned(), Arc::new(handle));

            Ok(())
        } else {
            Err(ThreadError::AlreadyExists)?
        }
    }

    #[inline(always)]
    async fn tokio_thread(
        &self,
        key: Arc<str>,
        server_id: Arc<str>,
        sv_license_key_token: Arc<str>,
        server_name: Arc<str>,
    ) -> JoinHandle<()> {
        let threads = self.threads.clone();
        let stock_repo = self.stock_repo.clone();
        let heartbeat = self.heartbeat.clone();
        let heartbeat_service = Arc::new(HeartbeatService::new());

        {
            let mut heartbeat = heartbeat.write().await;
            heartbeat.insert(key.to_owned().to_string(), Mutex::new(Instant::now()));
        }

        info!("Spawning thread for {}", server_name);

        tokio::spawn(async move {
            info!("Spawned thread for {}", server_name);
            let mut assigned_ids = AHashSet::new();

            loop {
                tokio::time::sleep(THREAD_SLEEP_TIME).await;
                let now = tokio::time::Instant::now();
                let new_assigned_players = Arc::new(Mutex::new(AHashSet::new()));

                {
                    let heartbeats = heartbeat.read().await;
                    let last_heartbeat = heartbeats.get(&*key).unwrap().lock().await;

                    if now.duration_since(*last_heartbeat).gt(&HEARTBEAT_TIMEOUT) {
                        drop(last_heartbeat);
                        drop(heartbeats);

                        info!("Thread {} timed out", server_name);
                        threads.write().await.remove(&*key).unwrap();

                        {
                            let mut heartbeat = heartbeat.write().await;
                            heartbeat.remove(&*key);
                        }

                        break;
                    }
                }

                let mut players = stock_repo.find_all_by_server(&server_id).await;

                {
                    players.retain(|player| {
                        if let Some(expire_on) = &player.expireOn {
                            if expire_on.lt(&chrono::Utc::now()) {
                                info!("Bot {} expired at {}", player.id.as_ref().unwrap(), expire_on);
                                return false;
                            }
                        }
                        true
                    });
                }

                let new_players = Arc::new(players);

                {
                    let mut new_assigned_players = new_assigned_players.lock().await;

                    for player in &*new_players {
                        if let Some(id) = &player.id {
                            if !assigned_ids.contains(id) {
                                new_assigned_players.insert(player.clone());
                            }

                            assigned_ids.insert(id.to_owned());
                        }
                    }

                    // if !new_assigned_players.is_empty() {
                    //     info!(
                    //         "New players assigned to {}: {:?}",
                    //         server_name,
                    //         new_assigned_players.len()
                    //     );
                    // }
                }

                let assigned_players_task = tokio::task::spawn({
                    let key = key.clone();
                    let sv_license_key_token = sv_license_key_token.clone();
                    let assigned_players = new_assigned_players.clone();
                    let heartbeat_service = heartbeat_service.clone();

                    async move {
                        stream::iter(assigned_players.lock().await.iter())
                            .for_each_concurrent(None, |player| {
                                let key = &key;
                                let sv_license_key_token = &sv_license_key_token;
                                let heartbeat_service = &heartbeat_service;

                                async move {
                                    if player.machineHash.is_some()
                                        && player.entitlementId.is_some()
                                    {
                                        let result = heartbeat_service
                                            .send_ticket(
                                                player.machineHash.as_ref().unwrap(),
                                                player.entitlementId.as_ref().unwrap(),
                                                sv_license_key_token,
                                            )
                                            .await;

                                        if let Err(error) = result {
                                            info!("Thread {} ticket error: {:?}", key, error);
                                        }
                                    }
                                }
                            })
                            .await
                    }
                });
                
                let key_clone = key.clone();
                let heartbeat_service_clone = heartbeat_service.clone();
                let new_players_task = tokio::task::spawn(async move {
                    if new_players.is_empty() {
                        println!("No new players");
                        return;
                    }
                
                    let key = key_clone;
                    let new_players_cloned = new_players.clone();
                    let heartbeat_service_cloned = heartbeat_service_clone;
                
                    stream::iter(&*new_players_cloned)
                        .for_each_concurrent(None, |player| {
                            let key = &key;
                            let heartbeat_service = &heartbeat_service_cloned;
                
                            async move {
                                if player.machineHash.is_some()
                                    && player.entitlementId.is_some()
                                {
                                    let result = heartbeat_service
                                        .send_entitlement(
                                            player.machineHash.as_ref().unwrap(),
                                            player.entitlementId.as_ref().unwrap(),
                                        )
                                        .await;
                
                                    if let Err(error) = result {
                                        info!("Thread {} heartbeat error: {:?}", key, error);
                                    }
                                }
                            }
                        })
                        .await;
                });
                
                let (t1, t2) = tokio::join!(assigned_players_task, new_players_task);
                if let Err(panic_info) = &t1 {
                    eprintln!("assigned_players_task panicked: {:?}", panic_info);
                }
                if let Err(panic_info) = &t2 {
                    eprintln!("new_players_task panicked: {:?}", panic_info);
                }
                t1.unwrap_or_default();
                t2.unwrap_or_default();

                info!(
                    "Thread {:20} took {:5}ms for {:5} bots",
                    server_name,
                    now.elapsed().as_millis(),
                    assigned_ids.len()
                );

                assigned_ids.clear();
            }
        })
    }
}
