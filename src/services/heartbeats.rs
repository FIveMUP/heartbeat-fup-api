use std::{sync::Arc, time::Duration};

use crate::{config::Database, error::ServerError};
use hyper::header::USER_AGENT;
use reqwest::header::{HeaderMap, CONTENT_TYPE};
use tracing::info;
#[derive(Clone)]
pub struct HeartbeatService {
    req_client_with_proxy: reqwest::Client,
    req_client_without_proxy: reqwest::Client,
}

impl HeartbeatService {
    pub fn new() -> Self {
        return Self {
            req_client_with_proxy: reqwest::Client::builder()
                .proxy(
                    reqwest::Proxy::all(
                        "http://customer-fivemup:FiveMUP2k23HappySex@dc.pr.oxylabs.io:10000",
                    )
                    .unwrap(),
                )
                .timeout(Duration::from_secs(8))
                .build()
                .unwrap(),
            req_client_without_proxy: reqwest::Client::builder()
                .timeout(Duration::from_secs(8))
                .build()
                .unwrap(),
        };
    }

    pub async fn send_entitlement(
        &self,
        machine_hash: &str,
        entitlement_id: &str,
    ) -> Result<bool, ServerError> {
        // info!("Sending entitlement for {}", machine_hash);

        let machine_hash_encoded = urlencoding::encode(machine_hash);
        let entitlement_heartbeat = format!(
            "entitlementId={}&f=%7b%7d&gameName=gta5&h2=YyMyxwNpROOEdyxjBu%2bNls1LHzPzx1zTEX7RtDmwD5Eb2MPVgeWNFbNZC3YfGgUnbriTU2jsl7jO0SQ9%2bmDqmU1rLf075r4bxMuKLjcUu2IPy3zVXd2ni2xVJJw8%2bFOoWqaTKIQGggBYEBEBRNOsFNjp6TLqbCwKiqMmc7rl8pLj6SCUm1MpNcBg%2fIE15VmMk4erFf26PdrA4GpAKAP%2fdsM9QaY1GbBnwM4V4xWl8EtLWFPF0XW9xePpm5ZPOjU3OfMAZ2eTF6cNkNsxAGHIMB4VTaKLGWoWmRToEEzbh9wTebY97mYeFdtqF8L%2bnNPVv6y0k4szAwdbInJ2oE73iFj5mZIKLGxqKtNGg9r10nJm2Bk1bTchSWTKlsI%2ffN1vvG6g1fxNDf5%2bJyqGnhktaEMt7L8JTxpgHPuAKtAN795kAM%2fZRgHUUqJzxnH4Ps3jSaMAt5eDpzfdkGvhADFIMMfSEEZ6WqQyvwRw85arnc6IgNYKFlqzGnpsHcWE13elDaRPbgNfMwT7U4Jk31vcfSsadYeqN6Ngad6CeF9zty7GWMklfWcRuaRqtiJvPI3%2fhGymZwPdFHsWvsBEFcbKTWVukjVzaXbuuOH81iY%2fCw7Mbq9A%2f%2fERGFNFW5HXUd9WCZsUooXHJcjVuczxO0BgQLfyEGaaemQSr0RwA3abTe7l5nY4wMC%2fJKkB1AKURTTsJcHhbK0Xrz14b5XOZIZDNlUGQpXweFTMWeualdOAxGUvDnnD0%2fqIZ39zjnPdulZUxCzGt%2fPt1Mt2nsAEJaYq%2fSLBqoahs9UtgGs%2fX9PAqqsnJdsRJ%2bZXKA%2fGfeBr58TCQsDJ8B1CCkqqsmAjItskmOY6w2%2fNGhQw7enImzXwvO4%3d&machineHash=AQAL&machineHashIndex={}&rosId=1234",
            entitlement_id,
            machine_hash_encoded
        );

        let mut headers = HeaderMap::new();

        headers.insert(
            CONTENT_TYPE,
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
        headers.insert(
            USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.71 Safari/537.3".parse().unwrap(),
        );

        let response = self
            .req_client_without_proxy
            .post("https://lambda.fivem.net/api/validate/entitlement")
            .headers(headers)
            .body(entitlement_heartbeat)
            .send()
            .await?;

        let response_status = response.status();
        if response_status.is_success() {
            Ok(true)
        } else {
            eprintln!(
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

        tokio::time::sleep(Duration::from_secs(1)).await;

        let ticket_heartbeat = format!(
            "gameName=gta5&guid=148618792012444134&machineHash=AQAL&machineHashIndex={}&server=http%3a%2f%51.91.102.108%3a30120%2f&serverKeyToken={}&token={}",
            urlencoding::encode(machine_hash),
            urlencoding::encode(sv_license_key_token),
            entitlement_id
        );

        // info!("Sending ticket for machine hash {}", machine_hash);

        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
        headers.insert(
            USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.71 Safari/537.3".parse().unwrap(),
        );

        let response = self
            .req_client_with_proxy
            .post("https://lambda.fivem.net/api/ticket/create")
            .headers(headers)
            .body(ticket_heartbeat)
            .send()
            .await?;

        if response.status().is_success() {
            // extract ticket from response json
            let response_json = response
                .text()
                .await
                .unwrap()
                .parse::<serde_json::Value>()
                .unwrap();
            let ticket = response_json["ticket"].as_str().unwrap_or("");

            if ticket.is_empty() {
                return Err(ServerError::NOT_FOUND);
            }
            return Ok(true);
        } else {
            return Err(ServerError::NOT_FOUND);
        }
    }
}
