use axum::Server;
use config::{init_tracing, Database};
use dotenvy::dotenv;
use mimalloc::MiMalloc;
use std::{net::TcpListener, sync::Arc};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod config;
mod controllers;
mod error;
mod response;
mod routes;
mod services;
mod states;

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_tracing();
    let db = Arc::new(Database::new().await);

    let listener = TcpListener::bind("0.0.0.0:9000").unwrap();

    Server::from_tcp(listener)
        .unwrap()
        .serve(routes::routes(db).await)
        .await
        .unwrap();
}
