use config::{init_tracing, Database};
use dotenvy::dotenv;
use mimalloc::MiMalloc;
use tokio::net::TcpListener;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod config;
mod controllers;
mod entities;
mod error;
mod repositories;
mod response;
mod routes;
mod services;
mod states;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_tracing();
    let db = Database::new().await;
    let listener = TcpListener::bind("0.0.0.0:9000").await.unwrap();

    axum::serve(listener, routes::routes(db).await)
        .await
        .unwrap();
}
