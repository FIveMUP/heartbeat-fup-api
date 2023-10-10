use crate::{config::Database, repositories::StockAccountRepository, services::HeartbeatService};
use ahash::{AHashMap, AHashSet};
use futures::{stream, StreamExt};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, RwLock},
    task::JoinHandle,
    time::Instant,
};
use tracing::info;

const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct ThreadService {
    db: Arc<Database>,
    threads: Arc<RwLock<AHashMap<String, Arc<JoinHandle<()>>>>>,
    heartbeat: Arc<Mutex<AHashMap<String, Instant>>>,
}

impl ThreadService {
    pub fn new(db: &Arc<Database>) -> Self {
        Self {
            db: db.clone(),
            threads: Arc::new(RwLock::new(AHashMap::new())),
            heartbeat: Arc::new(Mutex::new(AHashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> bool {
        self.threads.read().await.contains_key(key)
    }

    pub async fn heartbeat(&self, key: &str) {
        let mut heartbeat_map = self.heartbeat.lock().await;

        if let Some(heartbeat) = heartbeat_map.get_mut(key) {
            *heartbeat = Instant::now();
        }
    }

    pub async fn spawn_thread(&self, key: &str, server_id: &str, sv_license_key_token: &str, server_name: &str) {
        if !self.get(key).await {
            let handle = self.tokio_thread(
                Arc::from(key),
                Arc::from(server_id),
                Arc::from(sv_license_key_token),
                Arc::from(server_name),
            );

            self.threads
                .write()
                .await
                .insert(key.to_owned(), Arc::new(handle.await));
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
        let db = self.db.clone();
        let threads = self.threads.clone();
        let heartbeat = self.heartbeat.clone();
        let stock_repo = StockAccountRepository::new(&db);
        let heartbeat_service = Arc::new(HeartbeatService::new());

        self.heartbeat
            .lock()
            .await
            .insert(key.clone().to_string(), Instant::now());

        info!("Spawning thread for {}", server_name);

        tokio::spawn(async move {
            info!("Spawned thread for {}", server_name);
            let mut assigned_ids: Vec<String> = vec![];
            let new_ids = Arc::new(Mutex::new(AHashSet::new()));
            let new_assigned_players = Arc::new(Mutex::new(Vec::new()));

            loop {
                let now = tokio::time::Instant::now();
                let last_heartbeat = heartbeat.lock().await.get(&*key.clone()).copied().unwrap();

                if now.duration_since(last_heartbeat) > HEARTBEAT_TIMEOUT {
                    info!("Thread {} timed out", server_name);
                    threads.write().await.remove(&*key);
                    break;
                }

                let (mut new_ids, mut new_assigned_players, new_players) = tokio::join!(
                    async { new_ids.lock().await },
                    async { new_assigned_players.lock().await },
                    stock_repo.find_all_by_server(&server_id)
                );

                for player in &new_players {
                    if let Some(id) = &player.id {
                        if !assigned_ids.contains(id) {
                            new_assigned_players.push(player.clone());
                        }

                        // TODO: try to avoid cloning here
                        new_ids.insert(id.clone());
                    }
                }

                if !new_assigned_players.is_empty() {
                    info!(
                        "New players assigned to {}: {:?}",
                        server_name,
                        new_assigned_players.len()
                    );
                }

                assigned_ids = new_ids
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<String>>();

                let assigned_players_task = tokio::task::spawn({
                    let key: Arc<str> = key.clone();
                    let sv_license_key_token = sv_license_key_token.clone();
                    let assigned_players = new_assigned_players.clone();
                    let heartbeat_service = heartbeat_service.clone();

                    async move {
                        stream::iter(assigned_players)
                            .for_each_concurrent(None, |player| {
                                let key = key.clone();
                                let player = player.clone();
                                let sv_license_key_token = sv_license_key_token.clone();
                                let heartbeat_service = heartbeat_service.clone();

                                async move {
                                    if player.machineHash.is_some() && player.entitlementId.is_some() {
                                        let result = heartbeat_service
                                            .send_ticket(
                                                &player.machineHash.as_ref().unwrap(),
                                                &player.entitlementId.as_ref().unwrap(),
                                                &sv_license_key_token,
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

                let new_players_task = tokio::task::spawn({
                    let key = key.clone();
                    let new_players = new_players.clone();
                    let heartbeat_service = heartbeat_service.clone();

                    async move {
                        stream::iter(new_players)
                            .for_each_concurrent(None, |player| {
                                let key = key.clone();
                                let player = player.clone();
                                let heartbeat_service = heartbeat_service.clone();

                                async move {
                                    if player.machineHash.is_some() && player.entitlementId.is_some() {
                                        let result = heartbeat_service
                                            .send_entitlement(
                                                &player.machineHash.as_ref().unwrap(),
                                                &player.entitlementId.as_ref().unwrap(),
                                            )
                                            .await;

                                        if let Err(error) = result {
                                            info!("Thread {} heartbeat error: {:?}", key, error);
                                        }
                                    }
                                }
                            })
                            .await
                    }
                });

                let (_, _) = tokio::join!(assigned_players_task, new_players_task);

                new_ids.clear();
                new_assigned_players.clear();

                info!(
                    "Thread {:25} took {:5}ms for {:5} bots",
                    server_name,
                    now.elapsed().as_millis(),
                    assigned_ids.len()
                );

                tokio::time::sleep(Duration::from_secs(6)).await;
            }
        })
    }
}
