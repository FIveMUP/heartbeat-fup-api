use axum::Server;
use dotenvy::dotenv;
use std::net::TcpListener;

mod controllers;
mod error;
mod response;
mod routes;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let listener = TcpListener::bind("0.0.0.0:9000").unwrap();

    Server::from_tcp(listener)
        .unwrap()
        .serve(routes::routes().await)
        .await
        .unwrap();
}
