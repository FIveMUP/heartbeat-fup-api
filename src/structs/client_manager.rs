use std::ops::Deref;

use axum::async_trait;
use deadpool::managed::{self, Manager};
use once_cell::sync::Lazy;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client, Proxy,
};
use tokio::time::Duration;

use super::tracking_client::TrackingClient;

static HEADERS: Lazy<HeaderMap> = Lazy::new(|| {
    let mut headers = HeaderMap::with_capacity(1);

    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );

    headers
});

const PROXY_URL: &str = "http://customer-polini:Bigcipote69not96@dc.ca-pr.oxylabs.io:34000";

pub struct ClientManager;

#[async_trait]
impl Manager for ClientManager {
    type Type = TrackingClient;
    type Error = reqwest::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let client = Client::builder()
            .proxy(Proxy::all(PROXY_URL).unwrap())
            .user_agent("CitizenFX/1 (with adhesive; rel. 7194)")
            .default_headers(HEADERS.deref().clone())
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(TrackingClient::new(client))
    }

    async fn recycle(
        &self,
        client: &mut TrackingClient,
        _metrics: &managed::Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        if client.is_busy() {
            return Err(managed::RecycleError::StaticMessage("Client is busy"));
        }

        Ok(())
    }
}
