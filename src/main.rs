use config::init_tracing;
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
    let listener = TcpListener::bind("0.0.0.0:9000").await.unwrap();

    axum::serve(listener, routes::routes().await).await.unwrap();
}
