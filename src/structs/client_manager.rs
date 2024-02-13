use std::ops::Deref;

use axum::async_trait;
use deadpool::managed::{self, Manager, RecycleResult};
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

const TIMEOUT: Duration = Duration::from_secs(5);
const PROXY_URL: &str = "http://customer-polini:Bigcipote69not96@dc.ca-pr.oxylabs.io:34000";

// Allowing this because never mutates
#[allow(clippy::declare_interior_mutable_const)]
const USER_AGENT: HeaderValue = HeaderValue::from_static("CitizenFX/1 (with adhesive; rel. 7194)");

pub struct ClientManager;

#[async_trait]
impl Manager for ClientManager {
    type Type = TrackingClient;
    type Error = reqwest::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let client = Client::builder()
            .proxy(Proxy::all(PROXY_URL).unwrap())
            .user_agent(USER_AGENT)
            .default_headers(HEADERS.deref().clone())
            .timeout(TIMEOUT)
            .build()?;

        Ok(TrackingClient::new(client))
    }

    async fn recycle(
        &self,
        obj: &mut Self::Type,
        _metrics: &managed::Metrics,
    ) -> RecycleResult<Self::Error> {
        if !obj.is_recyclable() {
            return Err(managed::RecycleError::StaticMessage(
                "Client is not recyclable",
            ));
        }

        Ok(())
    }
}
