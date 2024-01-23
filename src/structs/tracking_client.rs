use reqwest::{Client, Error, Response};
use std::sync::atomic::{AtomicU8, Ordering};

pub struct TrackingClient {
    client: Client,
    active_requests: AtomicU8,
}

impl TrackingClient {
    #[inline]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            active_requests: AtomicU8::new(0),
        }
    }

    #[inline]
    pub async fn execute_post(&self, url: &str, body: String) -> Result<Response, Error> {
        self.active_requests.fetch_add(1, Ordering::SeqCst);

        let response = self.client.post(url).body(body).send().await;

        self.active_requests.fetch_sub(1, Ordering::SeqCst);

        response
    }

    #[inline]
    pub fn is_busy(&self) -> bool {
        self.active_requests.load(Ordering::SeqCst) >= 5
    }
}
