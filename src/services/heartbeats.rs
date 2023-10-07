use crate::error::{AppResult, ServerError};
use axum::http::HeaderValue;
use hyper::header::USER_AGENT;
use reqwest::{
    header::{HeaderMap, CONTENT_TYPE},
    Client, Proxy,
};
use std::time::Duration;
use tracing::error;

#[derive(Clone)]
pub struct HeartbeatService {
    req_client: Client,
}

impl HeartbeatService {
    pub fn new() -> Self {
        Self {
            req_client: Client::builder()
                .proxy(
                    Proxy::http(
                        "https://cristianrg36:Z36BXSuHWddm3QCm@proxy.packetstream.io:31111",
                    )
                    .unwrap(),
                )
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }
    pub async fn send_entitlement(
        &self,
        machine_hash: &str,
        entitlement_id: &str,
    ) -> AppResult<bool> {
        let machine_hash_encoded = urlencoding::encode(machine_hash);

        let response = self.req_client
            .post("https://lambda.fivem.net/api/validate/entitlement")
            .headers(
                HeaderMap::from_iter(
                    vec![
                        (
                            CONTENT_TYPE,
                            HeaderValue::from_static("application/x-www-form-urlencoded"),
                        ),
                        (
                            USER_AGENT,
                            HeaderValue::from_static(
                                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.71 Safari/537.3"
                            ),
                        )
                    ]
                )
            )
            .body(
                format!(
                    "entitlementId={entitlement_id}&f=%7b%7d&gameName=gta5&h2=YyMyxwNpROOEdyxjBu%2bNls1LHzPzx1zTEX7RtDmwD5Eb2MPVgeWNFbNZC3YfGgUnbriTU2jsl7jO0SQ9%2bmDqmU1rLf075r4bxMuKLjcUu2IPy3zVXd2ni2xVJJw8%2bFOoWqaTKIQGggBYEBEBRNOsFNjp6TLqbCwKiqMmc7rl8pLj6SCUm1MpNcBg%2fIE15VmMk4erFf26PdrA4GpAKAP%2fdsM9QaY1GbBnwM4V4xWl8EtLWFPF0XW9xePpm5ZPOjU3OfMAZ2eTF6cNkNsxAGHIMB4VTaKLGWoWmRToEEzbh9wTebY97mYeFdtqF8L%2bnNPVv6y0k4szAwdbInJ2oE73iFj5mZIKLGxqKtNGg9r10nJm2Bk1bTchSWTKlsI%2ffN1vvG6g1fxNDf5%2bJyqGnhktaEMt7L8JTxpgHPuAKtAN795kAM%2fZRgHUUqJzxnH4Ps3jSaMAt5eDpzfdkGvhADFIMMfSEEZ6WqQyvwRw85arnc6IgNYKFlqzGnpsHcWE13elDaRPbgNfMwT7U4Jk31vcfSsadYeqN6Ngad6CeF9zty7GWMklfWcRuaRqtiJvPI3%2fhGymZwPdFHsWvsBEFcbKTWVukjVzaXbuuOH81iY%2fCw7Mbq9A%2f%2fERGFNFW5HXUd9WCZsUooXHJcjVuczxO0BgQLfyEGaaemQSr0RwA3abTe7l5nY4wMC%2fJKkB1AKURTTsJcHhbK0Xrz14b5XOZIZDNlUGQpXweFTMWeualdOAxGUvDnnD0%2fqIZ39zjnPdulZUxCzGt%2fPt1Mt2nsAEJaYq%2fSLBqoahs9UtgGs%2fX9PAqqsnJdsRJ%2bZXKA%2fGfeBr58TCQsDJ8B1CCkqqsmAjItskmOY6w2%2fNGhQw7enImzXwvO4%3d&machineHash=AQAL&machineHashIndex={machine_hash_encoded}&rosId=1234"
                )
            )
            .send().await
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
    ) -> AppResult<bool> {
        self.send_entitlement(machine_hash, entitlement_id).await?;

        let ticket_heartbeat = format!(
            "gameName=gta5&guid=148618792012444134&machineHash=AQAL&machineHashIndex={}&server=http%3a%2f%51.91.102.108%3a30120%2f&serverKeyToken={}&token={}",
            urlencoding::encode(machine_hash),
            urlencoding::encode(sv_license_key_token),
            entitlement_id
        );

        let response = self.req_client
            .post("https://lambda.fivem.net/api/ticket/create")
            .headers(
                HeaderMap::from_iter(
                    vec![
                        (
                            CONTENT_TYPE,
                            HeaderValue::from_static("application/x-www-form-urlencoded"),
                        ),
                        (
                            USER_AGENT,
                            HeaderValue::from_static(
                                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.71 Safari/537.3"
                            ),
                        )
                    ]
                )
            )
            .body(ticket_heartbeat)
            .send().await
            .unwrap();

        if response.status().is_success() {
            Ok(true)
        } else {
            error!(
                "Failed to send ticket heartbeat. HTTP status: {}",
                response.status()
            );
            Err(ServerError::NotFound)?
        }
    }
}
