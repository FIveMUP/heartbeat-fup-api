use reqwest::{Client, Error, Response};
use std::sync::atomic::{AtomicU8, Ordering};

const MAX_RECYCLE: u8 = 3;

pub struct TrackingClient {
    client: Client,
    counter: AtomicU8,
}

impl TrackingClient {
    #[inline]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            counter: AtomicU8::new(0),
        }
    }

    #[inline]
    pub async fn execute_post(&self, url: &str, body: String) -> Result<Response, Error> {
        let response = self.client.post(url).body(body).send().await;
        self.counter.fetch_add(1, Ordering::Relaxed);

        response
    }

    #[inline]
    pub fn is_recyclable(&self) -> bool {
        self.counter.load(Ordering::SeqCst) < MAX_RECYCLE
    }
}
