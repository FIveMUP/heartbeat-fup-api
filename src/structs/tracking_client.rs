use reqwest::{Client, Error, Response};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct TrackingClient {
    client: Client,
    active_requests: AtomicBool,
}

impl TrackingClient {
    #[inline]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            active_requests: AtomicBool::new(false),
        }
    }

    #[inline]
    pub async fn execute_post(&self, url: &str, body: String) -> Result<Response, Error> {
        self.active_requests.store(true, Ordering::SeqCst);

        let response = self.client.post(url).body(body).send().await;

        self.active_requests.store(false, Ordering::SeqCst);

        response
    }

    #[inline]
    pub fn is_busy(&self) -> bool {
        self.active_requests.load(Ordering::SeqCst)
    }
}
