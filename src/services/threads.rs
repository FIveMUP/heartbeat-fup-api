use crate::{
    config::Database,
    entities::StockAccount,
    error::{AppResult, ThreadError},
    repositories::StockAccountRepository,
    services::FivemService,
    utils::lcm,
};
use ahash::{AHashMap, AHashSet};
use chrono::Utc;
use compact_str::CompactString;
use futures::{stream, StreamExt};
use parking_lot::RwLock;
use std::{sync::Arc, time::Duration};
use tokio::{
    task::JoinHandle,
    time::{self, Instant},
};
use tracing::{error, info, warn};

const UPDATE_PLAYERS_TICK: u8 = 2; // 2 * 60 = 120 seconds (2 minutes)
const UPDATE_EXPIRED_PLAYERS_TICK: u8 = 4; // 2 * 4 = 240 seconds (4 minutes)
const SHRINK_HASHES_TICK: u8 = 6; // 6 * 60 = 360 seconds (6 minutes)
const UPDATE_MAX_TICK: u8 = lcm(
    UPDATE_PLAYERS_TICK,
    lcm(UPDATE_EXPIRED_PLAYERS_TICK, SHRINK_HASHES_TICK),
);

const MIN_HEARTBEAT_TIME: Duration = Duration::from_secs(5);
const THREAD_SLEEP_TIME: Duration = Duration::from_secs(60);
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct ThreadService {
    fivem_service: &'static FivemService,
    stock_repo: &'static StockAccountRepository,
    heartbeat: &'static RwLock<AHashMap<CompactString, Instant>>,
    threads: &'static RwLock<AHashMap<CompactString, JoinHandle<()>>>,
}

impl ThreadService {
    pub fn new(db: &Database) -> Self {
        let fivem_service: &FivemService = Box::leak(Box::new(FivemService::new()));
        let stock_repo = Box::leak(Box::new(StockAccountRepository::new(db)));
        let threads = Box::leak(Box::new(RwLock::new(AHashMap::with_capacity(50))));
        let heartbeat = Box::leak(Box::new(RwLock::new(AHashMap::with_capacity(50))));

        Self {
            threads,
            fivem_service,
            heartbeat,
            stock_repo,
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
            threads.insert(key, handle);

            Ok(())
        } else {
            Err(ThreadError::AlreadyExists)?
        }
    }

    // todo: Use in a better way the locks
    #[inline(always)]
    async fn tokio_thread(
        &self,
        key: CompactString,
        server_id: CompactString,
        sv_license_key_token: CompactString,
        server_name: CompactString,
    ) -> JoinHandle<()> {
        let fivem_service = self.fivem_service;
        let stock_repo = self.stock_repo;
        let threads = self.threads;
        let heartbeat = self.heartbeat;

        {
            let mut heartbeat = heartbeat.write();
            heartbeat.insert(key.clone(), Instant::now());
        }

        info!("Spawning thread for {}", server_name);

        tokio::spawn(async move {
            let mut thread_startup = true;
            let mut update_counter = 0u8;
            let mut expired_ids = AHashSet::new();
            let sv_license_key_token: Arc<str> =
                Arc::from(urlencoding::encode(&sv_license_key_token));

            let players_count = stock_repo.get_count(&server_id).await.unwrap_or(20);
            let assigned_players = Arc::from(RwLock::new(AHashMap::with_capacity(players_count)));
            let new_players: Arc<RwLock<AHashMap<CompactString, StockAccount>>> =
                Arc::from(RwLock::new(AHashMap::with_capacity(players_count)));

            info!("Spawned thread for {}", server_name);

            loop {
                time::sleep(THREAD_SLEEP_TIME).await;
                let now = Instant::now();

                // Check Heartbeat Timeout
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

                let mut new_players_vec = Vec::new();

                // Update Players Hashes
                {
                    let mut new_players = new_players.write_arc();
                    let mut assigned_players = assigned_players.write_arc();

                    // Check if there are new players
                    if !new_players.is_empty() {
                        for (id, player) in new_players.iter() {
                            if assigned_players.contains_key(id) {
                                continue;
                            }

                            assigned_players.insert(id.to_owned(), player.to_owned());
                        }

                        new_players.retain(|id, _player| !assigned_players.contains_key(id));
                    }

                    if update_counter & UPDATE_PLAYERS_TICK == 0 {
                        let Ok(db_players) = stock_repo.find_all_by_server(&server_id).await else {
                            error!("Thread: {}, got a Database error", server_name);
                            break;
                        };

                        let time = Utc::now();
                        let update_expired = update_counter & UPDATE_EXPIRED_PLAYERS_TICK == 0;

                        if update_expired && expired_ids.len() > 0 {
                            expired_ids.drain();
                        }

                        for (id, player) in db_players.iter() {
                            if expired_ids.contains(id) {
                                continue;
                            }

                            // Expiration Checks
                            if update_expired && time > player.expire_on {
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
                                let id = id.to_owned();
                                let player = player.to_owned();

                                new_players_vec.push(player.clone());
                                new_players.insert(id, player);
                            }
                        }

                        if update_expired && expired_ids.len() == db_players.len() {
                            error!("Thread: {}, all players expired", server_name);
                            return;
                        }

                        assigned_players.retain(|id, _player| db_players.contains_key(id));
                    }
                }

                // New players
                let new_players_task = tokio::task::spawn({
                    let sv_license_key_token = sv_license_key_token.clone();
                    let cloned_new_players = new_players.clone();

                    async move {
                        if new_players_vec.is_empty() {
                            return;
                        }

                        stream::iter(new_players_vec.iter())
                            .for_each_concurrent(None, |player| {
                                let sv_license_key_token = sv_license_key_token.clone();
                                let cloned_new_players = cloned_new_players.clone();

                                async move {
                                    let result = fivem_service
                                        .initialize_player(
                                            &player.machine_hash,
                                            &player.entitlement_id,
                                            &player.account_index,
                                            &sv_license_key_token,
                                        )
                                        .await;

                                    if let Err(error) = result {
                                        warn!(
                                            "Error Sending Entitlement for Player {}: {:?}",
                                            &player.id, error
                                        );

                                        // Todo: Trying to avoid an write lock attack
                                        cloned_new_players.write().remove(&player.id);
                                    }
                                }
                            })
                            .await;
                    }
                });

                // Assigned players
                let assigned_players_task = tokio::task::spawn({
                    let assigned_players = assigned_players.clone();

                    async move {
                        let assigned_players = assigned_players.read_arc();

                        if assigned_players.is_empty() {
                            return;
                        }

                        stream::iter(assigned_players.values())
                            .for_each_concurrent(None, |player| async move {
                                let result = fivem_service
                                    .heartbeat(
                                        &player.machine_hash,
                                        &player.entitlement_id,
                                        &player.account_index,
                                    )
                                    .await;

                                if let Err(error) = result {
                                    info!("Player {} ticket error: {:?}", &player.id, error);
                                }
                            })
                            .await
                    }
                });

                if let Err(e) = tokio::try_join!(new_players_task, assigned_players_task) {
                    error!("Thread {} error: {:?}", server_name, e);
                };

                // Todo: see if this is the best way to do to this
                let mut new_players = new_players.write();
                let mut assigned_players = assigned_players.write();

                info!(
                    "Server Thread :{:20}, With :{:5} Players :{:5}Ms",
                    server_name,
                    new_players.len() + assigned_players.len(),
                    now.elapsed().as_millis()
                );

                if thread_startup {
                    thread_startup = false;
                    new_players.shrink_to_fit();
                }

                if !thread_startup && update_counter & SHRINK_HASHES_TICK == 0 {
                    new_players.shrink_to_fit();
                    assigned_players.shrink_to_fit();
                }

                update_counter = (update_counter + 1) % UPDATE_MAX_TICK;
            }
        })
    }
}
