use crate::error::ServerError;
use axum::{extract::FromRef, http::HeaderValue};
use hyper::header::USER_AGENT;
use once_cell::sync::Lazy;
use reqwest::{
    header::{HeaderMap, CONTENT_TYPE},
    Client, Proxy,
};
use std::time::Duration;
use tracing::error;

const FIVEM_URL: &str = "https://lambda.fivem.net/api";
const PROXY_URL: &str = "http://customer-fivemup:FiveMUP2k23HappySex@dc.pr.oxylabs.io:10000";
static HEADERS: Lazy<HeaderMap> = Lazy::new(|| {
    HeaderMap::from_iter(
        vec![
            (CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded")),
            (
                USER_AGENT,
                HeaderValue::from_static(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.71 Safari/537.3"
                ),
            )
        ]
    )
});

#[derive(Clone)]
pub struct HeartbeatService {
    req_client_with_proxy: Client,
    req_client_without_proxy: Client,
}

impl HeartbeatService {
    pub fn new() -> Self {
        Self {
            req_client_with_proxy: Client::builder()
                .proxy(Proxy::all(PROXY_URL).unwrap())
                .timeout(Duration::from_secs(8))
                .build()
                .unwrap(),

            req_client_without_proxy: Client::builder()
                .timeout(Duration::from_secs(8))
                .build()
                .unwrap(),
        }
    }

    pub async fn send_entitlement(
        &self,
        machine_hash: &str,
        entitlement_id: &str,
    ) -> Result<bool, ServerError> {
        let machine_hash_encoded = urlencoding::encode(machine_hash);
        let entitlement_heartbeat = format!(
            "entitlementId={}&f=%7b%7d&gameName=gta5&h2=YyMyxwNpROOEdyxjBu%2bNls1LHzPzx1zTEX7RtDmwD5Eb2MPVgeWNFbNZC3YfGgUnbriTU2jsl7jO0SQ9%2bmDqmU1rLf075r4bxMuKLjcUu2IPy3zVXd2ni2xVJJw8%2bFOoWqaTKIQGggBYEBEBRNOsFNjp6TLqbCwKiqMmc7rl8pLj6SCUm1MpNcBg%2fIE15VmMk4erFf26PdrA4GpAKAP%2fdsM9QaY1GbBnwM4V4xWl8EtLWFPF0XW9xePpm5ZPOjU3OfMAZ2eTF6cNkNsxAGHIMB4VTaKLGWoWmRToEEzbh9wTebY97mYeFdtqF8L%2bnNPVv6y0k4szAwdbInJ2oE73iFj5mZIKLGxqKtNGg9r10nJm2Bk1bTchSWTKlsI%2ffN1vvG6g1fxNDf5%2bJyqGnhktaEMt7L8JTxpgHPuAKtAN795kAM%2fZRgHUUqJzxnH4Ps3jSaMAt5eDpzfdkGvhADFIMMfSEEZ6WqQyvwRw85arnc6IgNYKFlqzGnpsHcWE13elDaRPbgNfMwT7U4Jk31vcfSsadYeqN6Ngad6CeF9zty7GWMklfWcRuaRqtiJvPI3%2fhGymZwPdFHsWvsBEFcbKTWVukjVzaXbuuOH81iY%2fCw7Mbq9A%2f%2fERGFNFW5HXUd9WCZsUooXHJcjVuczxO0BgQLfyEGaaemQSr0RwA3abTe7l5nY4wMC%2fJKkB1AKURTTsJcHhbK0Xrz14b5XOZIZDNlUGQpXweFTMWeualdOAxGUvDnnD0%2fqIZ39zjnPdulZUxCzGt%2fPt1Mt2nsAEJaYq%2fSLBqoahs9UtgGs%2fX9PAqqsnJdsRJ%2bZXKA%2fGfeBr58TCQsDJ8B1CCkqqsmAjItskmOY6w2%2fNGhQw7enImzXwvO4%3d&machineHash=AQAL&machineHashIndex={}&rosId=1234",
            entitlement_id,
            machine_hash_encoded
        );

        let response = self
            .req_client_without_proxy
            .post(format!("{FIVEM_URL}/validate/entitlement"))
            .headers(HeaderMap::from_ref(&HEADERS))
            .body(entitlement_heartbeat)
            .send()
            .await
            .unwrap();

        let response_status = response.status();
        if response_status.is_success() {
            Ok(true)
        } else {
            error!(
                "Failed to send entitlement heartbeat. HTTP status: {}",
                response_status
            );

            Ok(false)
        }
    }

    pub async fn send_ticket(
        &self,
        machine_hash: &str,
        entitlement_id: &str,
        sv_license_key_token: &str,
    ) -> Result<bool, ServerError> {
        self.send_entitlement(machine_hash, entitlement_id).await?;

        let ticket_heartbeat = format!(
            "gameName=gta5&guid=148618792012444134&machineHash=AQAL&machineHashIndex={}&server=http%3a%2f%51.91.102.108%3a30120%2f&serverKeyToken={}&token={}",
            urlencoding::encode(machine_hash),
            urlencoding::encode(sv_license_key_token),
            entitlement_id
        );

        let response = self
            .req_client_with_proxy
            .post(format!("{FIVEM_URL}/ticket/create"))
            .headers(HeaderMap::from_ref(&HEADERS))
            .body(ticket_heartbeat)
            .send()
            .await
            .unwrap();

        if response.status().is_success() {
            let response_json = response.json::<serde_json::Value>().await.unwrap();

            if response_json["ticket"].is_null() {
                Err(ServerError::NotFound)?;
            }

            Ok(true)
        } else {
            Err(ServerError::NotFound)?
        }
    }
}
