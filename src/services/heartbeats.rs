use crate::error::{AppResult, CfxApiError};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client, Proxy,
};
use std::time::Duration;

const FIVEM_URL: &str = "https://lambda.fivem.net/api";
const EID_URL: &str = "https://cnl-hb-live.fivem.net/api";
const PROXY_URL: &str = "http://sp7j5w5bze:proxypassxd1234fivemup@eu.dc.smartproxy.com:20000";

#[derive(Clone)]
pub struct HeartbeatService {
    client: Client,
}

impl HeartbeatService {
    pub fn new() -> Self {
        let mut headers = HeaderMap::with_capacity(1);

        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        Self {
            client: Client::builder()
                .proxy(Proxy::all(PROXY_URL).unwrap())
                .user_agent("CitizenFX/1 (with adhesive; rel. 7194)")
                .default_headers(headers)
                .timeout(Duration::from_secs(15))
                .build()
                .unwrap(),
        }
    }

    pub async fn send_entitlement(
        &self,
        machine_hash: &str,
        entitlement_id: &str,
        account_index: &str,
    ) -> AppResult<()> {
        let response = {
            let entitlement_heartbeat = format!(
                "entitlementId={}&f=%7b%7d&gameName=gta5&h2=YyMyxwNpROOEdyxjBu%2bNls1LHzPzx1zTEX7RtDmwD5Eb2MPVgeWNFbNZC3YfGgUnbriTU2jsl7jO0SQ9%2bmDqmU1rLf075r4bxMuKLjcUu2IPy3zVXd2ni2xVJJw8%2bFOoWqaTKIQGggBYEBEBRNOsFNjp6TLqbCwKiqMmc7rl8pLj6SCUm1MpNcBg%2fIE15VmMk4erFf26PdrA4GpAKAP%2fdsM9QaY1GbBnwM4V4xWl8EtLWFPF0XW9xePpm5ZPOjU3OfMAZ2eTF6cNkNsxAGHIMB4VTaKLGWoWmRToEEzbh9wTebY97mYeFdtqF8L%2bnNPVv6y0k4szAwdbInJ2oE73iFj5mZIKLGxqKtNGg9r10nJm2Bk1bTchSWTKlsI%2ffN1vvG6g1fxNDf5%2bJyqGnhktaEMt7L8JTxpgHPuAKtAN795kAM%2fZRgHUUqJzxnH4Ps3jSaMAt5eDpzfdkGvhADFIMMfSEEZ6WqQyvwRw85arnc6IgNYKFlqzGnpsHcWE13elDaRPbgNfMwT7U4Jk31vcfSsadYeqN6Ngad6CeF9zty7GWMklfWcRuaRqtiJvPI3%2fhGymZwPdFHsWvsBEFcbKTWVukjVzaXbuuOH81iY%2fCw7Mbq9A%2f%2fERGFNFW5HXUd9WCZsUooXHJcjVuczxO0BgQLfyEGaaemQSr0RwA3abTe7l5nY4wMC%2fJKkB1AKURTTsJcHhbK0Xrz14b5XOZIZDNlUGQpXweFTMWeualdOAxGUvDnnD0%2fqIZ39zjnPdulZUxCzGt%2fPt1Mt2nsAEJaYq%2fSLBqoahs9UtgGs%2fX9PAqqsnJdsRJ%2bZXKA%2fGfeBr58TCQsDJ8B1CCkqqsmAjItskmOY6w2%2fNGhQw7enImzXwvO4%3d&i={}&machineHash=AQAL&machineHashIndex={}&rosId=1234",
                entitlement_id,
                account_index,
                machine_hash
            );

            self.client
                .post(format!("{EID_URL}/validate/entitlement"))
                .body(entitlement_heartbeat)
                .send()
                .await
                .map_err(|e| {
                    tracing::error!("Error sending entitlement heartbeat: {}", e);
                    CfxApiError::EntitlementHeartbeatFailed
                })?
                .status()
        };

        if !response.is_success() {
            println!("Entitlement heartbeat failed: {}", response);
            Err(CfxApiError::StatusCodeNot200)?
        }

        Ok(())
    }

    pub async fn send_ticket(
        &self,
        machine_hash: &str,
        entitlement_id: &str,
        account_index: &str,
        sv_license_key_token: &str,
    ) -> AppResult<()> {
        let response = {
            let ticket_heartbeat = format!(
                "gameName=gta5&guid=148618792012444134&machineHash=AQAL&machineHashIndex={}&server=http%3a%2f%51.91.102.108%3a30120%2f&serverKeyToken={}&token={}&i={}",
                machine_hash,
                urlencoding::encode(sv_license_key_token),
                entitlement_id,
                account_index
            );

            self.client
                .post(format!("{FIVEM_URL}/ticket/create"))
                .body(ticket_heartbeat)
                .send()
                .await
                .map_err(|_| CfxApiError::TicketHeartbeatFailed)?
        };

        self.send_entitlement(machine_hash, entitlement_id, account_index)
            .await?;

        if !response.status().is_success() {
            println!("Ticket heartbeat failed: {}", response.status());
            Err(CfxApiError::StatusCodeNot200)?
        }

        let response_json = response.json::<serde_json::Value>().await.unwrap();
        if response_json["ticket"].is_null() {
            Err(CfxApiError::TicketResponseNull)?;
        }

        Ok(())
    }
}
