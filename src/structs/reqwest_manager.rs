use axum::async_trait;
use deadpool::managed::{self, Manager};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client, Proxy,
};
use tokio::time::Duration;

const PROXY_URL: &str = "http://sp7j5w5bze:proxypassxd1234fivemup@eu.dc.smartproxy.com:20000";

pub struct ClientManager;

#[async_trait]
impl Manager for ClientManager {
    type Type = Client;
    type Error = reqwest::Error;

    async fn create(&self) -> Result<Client, Self::Error> {
        let mut headers = HeaderMap::with_capacity(1);

        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        Client::builder()
            .proxy(Proxy::all(PROXY_URL).unwrap())
            .user_agent("CitizenFX/1 (with adhesive; rel. 7194)")
            .default_headers(headers)
            .timeout(Duration::from_secs(10))
            .build()
    }

    async fn recycle(
        &self,
        _client: &mut Client,
        _: &managed::Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        Ok(())
        // client.get("https://google.com").send().is_ok()
    }
}
