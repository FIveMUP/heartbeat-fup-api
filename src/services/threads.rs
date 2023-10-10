use crate::{config::Database, repositories::StockAccountRepository, services::HeartbeatService};
use ahash::{AHashMap, AHashSet};
use futures::{stream, StreamExt};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{sync::RwLock, task::JoinHandle, time::Instant};
use tracing::info;

const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(30);

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

    pub fn heartbeat(&self, key: &str) {
        let mut heartbeat_map = self.heartbeat.lock().unwrap();

        if let Some(heartbeat) = heartbeat_map.get_mut(key) {
            info!("Heartbeat received for {}", key);
            *heartbeat = Instant::now();
        }
    }

    pub async fn spawn_thread(&self, key: &str, server_id: &str, sv_license_key_token: &str) {
        if !self.get(key).await {
            let handle = self.tokio_thread(
                Arc::from(key),
                Arc::from(server_id),
                Arc::from(sv_license_key_token),
            );

            self.threads
                .write()
                .await
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
        let threads = self.threads.clone();
        let heartbeat = self.heartbeat.clone();
        let stock_repo = StockAccountRepository::new(&db);
        let heartbeat_service = HeartbeatService::new();

        self.heartbeat
            .lock()
            .unwrap()
            .insert(key.clone().to_string(), Instant::now());

        info!("Spawning thread for {}", key);

        tokio::spawn(async move {
            info!("Spawned thread for {}", key);
            let mut assigned_ids: Vec<_> = vec![];

            loop {
                let last_heartbeat = heartbeat.lock().unwrap().get(&*key).copied().unwrap();
                tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;
                let now = tokio::time::Instant::now();

                let new_players = stock_repo.find_all_by_server(&server_id).await;

                let mut new_assigned_players = Vec::new();
                let mut new_ids = AHashSet::new();

                for player in &new_players {
                    if let Some(id) = &player.id {
                        new_ids.insert(id.clone());
                        if !assigned_ids.contains(id) {
                            new_assigned_players.push(player.clone());
                        }
                    }
                }

                if !new_assigned_players.is_empty() {
                    info!(
                        "New players assigned to {}: {:?}",
                        key,
                        new_assigned_players.len()
                    );
                }

                assigned_ids = new_ids.into_iter().collect();

                stream::iter(new_assigned_players)
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

                            if let Err(error) = result {
                                info!("Thread {} ticket error: {:?}", key, error);
                            }
                        }
                    })
                    .await;

                stream::iter(new_players)
                    .for_each_concurrent(None, |player| {
                        let key = key.clone();
                        let machine_hash = player.machineHash.clone();
                        let entitlement_id = player.entitlementId.clone();
                        let heartbeat_service = heartbeat_service.clone();

                        async move {
                            let result = heartbeat_service
                                .send_entitlement(
                                    &machine_hash.unwrap(),
                                    &entitlement_id.unwrap(),
                                )
                                .await;

                            if let Err(error) = result {
                                info!("Thread {} heartbeat error: {:?}", key, error);
                            }
                        }
                    })
                    .await;

                if now.duration_since(last_heartbeat).gt(&HEARTBEAT_TIMEOUT) {
                    threads.write().await.remove(&*key);
                    info!("Thread {} timed out, killing", key);
                    break;
                }

                info!(
                    "Thread {} took {}ms for {} bots",
                    key,
                    now.elapsed().as_millis(),
                    assigned_ids.len()
                );
            }
        })
    }
}
