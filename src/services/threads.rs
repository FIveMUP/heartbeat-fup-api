use crate::{
    config::Database,
    error::{AppResult, ThreadError},
    repositories::StockAccountRepository,
    services::HeartbeatService,
};
use ahash::{AHashMap, AHashSet};
use chrono::Utc;
use compact_str::CompactString;
use futures::{stream, StreamExt};
use parking_lot::RwLock;
use std::{sync::Arc, time::Duration};
use tokio::{task::JoinHandle, time::Instant};
use tracing::{error, info, warn};

const UPDATE_PLAYERS_TICK: u8 = 4; // 4 * 7 = 28 Seconds
const UPDATE_EXPIRED_PLAYERS_TICK: u8 = 8; // 4 * 4 * 7 = 112 seconds (1.8 minutes)
const UPDATE_MAX_TICK: u8 = UPDATE_PLAYERS_TICK * UPDATE_EXPIRED_PLAYERS_TICK; // If this number increase should use lcm instead of multiplication

const MIN_HEARTBEAT_TIME: Duration = Duration::from_secs(5);
const THREAD_SLEEP_TIME: Duration = Duration::from_secs(7);
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct ThreadService {
    stock_repo: Arc<StockAccountRepository>,
    heartbeat: Arc<RwLock<AHashMap<CompactString, Instant>>>,
    threads: Arc<RwLock<AHashMap<CompactString, Arc<JoinHandle<()>>>>>,
}

impl ThreadService {
    pub fn new(db: &Database) -> Self {
        Self {
            threads: Arc::new(RwLock::new(AHashMap::with_capacity(100))),
            heartbeat: Arc::new(RwLock::new(AHashMap::with_capacity(100))),
            stock_repo: Arc::new(StockAccountRepository::new(db)),
        }
    }

    pub fn get(&self, key: &str) -> bool {
        self.threads.read().contains_key(key)
    }

    pub fn heartbeat(&self, key: CompactString) -> AppResult<()> {
        let mut heartbeats = self.heartbeat.upgradable_read();

        if let Some(instant) = heartbeats.get(&key) {
            // Trying to avoid an RwLock attack
            if instant.elapsed() < MIN_HEARTBEAT_TIME {
                Err(ThreadError::HeartbeatTooHigh)?
            }

            heartbeats.with_upgraded(|heartbeats| {
                heartbeats.insert(key, Instant::now());
            })
        } else {
            Err(ThreadError::NotFound)?
        }

        Ok(())
    }

    pub async fn spawn_thread(
        &self,
        key: CompactString,
        server_id: CompactString,
        sv_license_key_token: CompactString,
        server_name: CompactString,
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
        key: CompactString,
        server_id: CompactString,
        sv_license_key_token: CompactString,
        server_name: CompactString,
    ) -> JoinHandle<()> {
        let stock_repo = self.stock_repo.clone();
        let threads = self.threads.clone();
        let heartbeat = self.heartbeat.clone();
        let heartbeat_service = HeartbeatService::new();

        {
            let mut heartbeat = heartbeat.write();
            heartbeat.insert(key.clone(), Instant::now());
        }

        info!("Spawning thread for {}", server_name);

        tokio::spawn(async move {
            info!("Spawned thread for {}", server_name);
            let mut update_counter = 0u8;
            let sv_license_key_token: Arc<str> = Arc::from(sv_license_key_token.to_string());
            let expired_ids = Arc::from(RwLock::new(AHashSet::new()));
            let new_players = Arc::from(RwLock::new(AHashMap::new()));
            let assigned_players = Arc::from(RwLock::new(AHashMap::new()));

            loop {
                tokio::time::sleep(THREAD_SLEEP_TIME).await;
                let now = Instant::now();

                {
                    let mut heartbeats = heartbeat.upgradable_read();
                    let last_heartbeat = heartbeats.get(&key).unwrap();

                    if now.duration_since(*last_heartbeat) > HEARTBEAT_TIMEOUT {
                        threads.write().remove(&key);

                        heartbeats.with_upgraded(|heartbeats| {
                            heartbeats.remove(&key);
                        });

                        info!("Thread {} timed out", server_name);
                        return;
                    }
                }

                if update_counter & UPDATE_PLAYERS_TICK == 0 {
                    let Ok(db_players) = stock_repo.find_all_by_server(&server_id).await else {
                        error!("Thread: {}, got a Database error", server_name);
                        break;
                    };

                    let time = Utc::now();
                    let should_update_expired_players =
                        update_counter & UPDATE_EXPIRED_PLAYERS_TICK == 0;
                    let mut expired_ids = expired_ids.write();
                    let mut new_players = new_players.write();
                    let mut assigned_players = assigned_players.write();

                    if should_update_expired_players && expired_ids.len() > 0 {
                        expired_ids.drain();
                    }

                    for (id, player) in db_players.iter() {
                        if expired_ids.contains(id) {
                            continue;
                        }

                        if should_update_expired_players && time > player.expire_on {
                            expired_ids.insert(id.to_owned());

                            match assigned_players.contains_key(id) {
                                true => {
                                    assigned_players.remove(id);
                                }

                                false => {
                                    if new_players.contains_key(id) {
                                        new_players.remove(id);
                                    }
                                }
                            }
                            continue;
                        }

                        if !assigned_players.contains_key(id) {
                            match new_players.contains_key(id) {
                                true => {
                                    assigned_players.insert(id.to_owned(), player.to_owned());
                                }

                                false => {
                                    new_players.insert(id.to_owned(), player.to_owned());
                                }
                            }
                        }
                    }

                    if should_update_expired_players && expired_ids.len() == db_players.len() {
                        error!("Thread: {}, all players expired", server_name);
                        break;
                    }

                    assigned_players.retain(|id, _player| db_players.contains_key(id));
                }

                // New players
                let new_players_task = tokio::task::spawn({
                    let sv_license_key_token = sv_license_key_token.clone();
                    let heartbeat_service = heartbeat_service.clone();
                    let cloned_new_players = new_players.clone();

                    async move {
                        let new_players = cloned_new_players.read_arc();

                        if new_players.is_empty() {
                            return;
                        }

                        stream::iter(new_players.values())
                            .for_each_concurrent(None, |player| {
                                let sv_license_key_token = sv_license_key_token.clone();
                                let heartbeat_service = heartbeat_service.clone();
                                let cloned_new_players = cloned_new_players.clone();

                                async move {
                                    let result = heartbeat_service
                                        .send_entitlement(
                                            &player.machine_hash,
                                            &player.entitlement_id,
                                            &player.account_index,
                                        )
                                        .await;

                                    if let Err(error) = result {
                                        warn!(
                                            "Error Sending Entitlement for Player {}: {:?}",
                                            &player.id, error
                                        );

                                        // Remove player from new_players if entitlement is invalid
                                        cloned_new_players.write().remove(&player.id);
                                        return;
                                    }

                                    let result = heartbeat_service
                                        .send_ticket(
                                            &player.machine_hash,
                                            &player.entitlement_id,
                                            &player.account_index,
                                            &sv_license_key_token,
                                        )
                                        .await;

                                    if let Err(error) = result {
                                        warn!(
                                            "Error Sending First Ticket for Player {}: {:?}",
                                            &player.id, error
                                        );
                                    }
                                }
                            })
                            .await;
                    }
                });

                // Assigned players
                let assigned_players_task = tokio::task::spawn({
                    let sv_license_key_token = sv_license_key_token.clone();
                    let heartbeat_service = heartbeat_service.clone();
                    let assigned_players = assigned_players.clone();

                    async move {
                        let assigned_players = assigned_players.read_arc();

                        if assigned_players.is_empty() {
                            return;
                        }

                        stream::iter(assigned_players.values())
                            .for_each_concurrent(None, |player| {
                                let sv_license_key_token = sv_license_key_token.clone();
                                let heartbeat_service = heartbeat_service.clone();

                                async move {
                                    let result = heartbeat_service
                                        .send_ticket(
                                            &player.machine_hash,
                                            &player.entitlement_id,
                                            &player.account_index,
                                            &sv_license_key_token,
                                        )
                                        .await;

                                    if let Err(error) = result {
                                        info!("Player {} ticket error: {:?}", &player.id, error);
                                    }
                                }
                            })
                            .await
                    }
                });

                if let Err(e) = tokio::try_join!(new_players_task, assigned_players_task) {
                    error!("Thread {} error: {:?}", server_name, e);
                };

                let bots = assigned_players.read().len();
                info!(
                    "Thread {:20} took {:5}ms for {:5} bots",
                    server_name,
                    now.elapsed().as_millis(),
                    bots
                );

                new_players.write().clear();
                update_counter = (update_counter + 1) % UPDATE_MAX_TICK;
            }
        })
    }
}
