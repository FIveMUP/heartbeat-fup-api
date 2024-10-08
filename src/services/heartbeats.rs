use std::{ops::Deref, time::Duration};

use crate::error::{AppResult, CfxApiError};
use once_cell::sync::Lazy;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client, Proxy,
};

const TIMEOUT: Duration = Duration::from_secs(5);
const TICKET_CREATION_URL: &str = "https://lambda.fivem.net/api/ticket/create";
const HEARTBEAT_URL: &str = "https://cnl-hb-live.fivem.net/api/validate/entitlement";
const PROXY_URL: &str = "http://customer-polini:Bigcipote69not96@dc.ca-pr.oxylabs.io:34000";

#[allow(clippy::declare_interior_mutable_const)]
const USER_AGENT: HeaderValue = HeaderValue::from_static("CitizenFX/1 (with adhesive; rel. 7194)");

static HEADERS: Lazy<HeaderMap> = Lazy::new(|| {
    let mut headers = HeaderMap::with_capacity(1);

    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );

    headers
});

#[derive(Clone)]
pub struct FivemService {
    client: Client,
}

impl FivemService {
    pub fn new() -> Self {
        let client = Client::builder()
            .proxy(Proxy::all(PROXY_URL).unwrap())
            .user_agent(USER_AGENT)
            .default_headers(HEADERS.deref().clone())
            .timeout(TIMEOUT)
            .build()
            .unwrap();

        Self { client }
    }

    #[inline(always)]
    pub async fn heartbeat(
        &self,
        machine_hash: &str,
        entitlement_id: &str,
        account_index: &str,
    ) -> AppResult<()> {
        let response = {
            let entitlement_heartbeat = [
                "entitlementId=",
                entitlement_id,
                "&f=%7b%7d&gameName=gta5&h2=YyMyxwNpROOEdyxjBu%2bNls1LHzPzx1zTEX7RtDmwD5Eb2MPVgeWNFbNZC3YfGgUnbriTU2jsl7jO0SQ9%2bmDqmU1rLf075r4bxMuKLjcUu2IPy3zVXd2ni2xVJJw8%2bFOoWqaTKIQGggBYEBEBRNOsFNjp6TLqbCwKiqMmc7rl8pLj6SCUm1MpNcBg%2fIE15VmMk4erFf26PdrA4GpAKAP%2fdsM9QaY1GbBnwM4V4xWl8EtLWFPF0XW9xePpm5ZPOjU3OfMAZ2eTF6cNkNsxAGHIMB4VTaKLGWoWmRToEEzbh9wTebY97mYeFdtqF8L%2bnNPVv6y0k4szAwdbInJ2oE73iFj5mZIKLGxqKtNGg9r10nJm2Bk1bTchSWTKlsI%2ffN1vvG6g1fxNDf5%2bJyqGnhktaEMt7L8JTxpgHPuAKtAN795kAM%2fZRgHUUqJzxnH4Ps3jSaMAt5eDpzfdkGvhADFIMMfSEEZ6WqQyvwRw85arnc6IgNYKFlqzGnpsHcWE13elDaRPbgNfMwT7U4Jk31vcfSsadYeqN6Ngad6CeF9zty7GWMklfWcRuaRqtiJvPI3%2fhGymZwPdFHsWvsBEFcbKTWVukjVzaXbuuOH81iY%2fCw7Mbq9A%2f%2fERGFNFW5HXUd9WCZsUooXHJcjVuczxO0BgQLfyEGaaemQSr0RwA3abTe7l5nY4wMC%2fJKkB1AKURTTsJcHhbK0Xrz14b5XOZIZDNlUGQpXweFTMWeualdOAxGUvDnnD0%2fqIZ39zjnPdulZUxCzGt%2fPt1Mt2nsAEJaYq%2fSLBqoahs9UtgGs%2fX9PAqqsnJdsRJ%2bZXKA%2fGfeBr58TCQsDJ8B1CCkqqsmAjItskmOY6w2%2fNGhQw7enImzXwvO4%3d&i=",
                account_index,
                "&machineHash=AQAL&machineHashIndex=",
                machine_hash,
                "&rosId=1234"
            ].concat();

            self.client
                .post(HEARTBEAT_URL)
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

    #[inline(always)]
    pub async fn insert_bot(
        &self,
        machine_hash: &str,
        entitlement_id: &str,
        account_index: &str,
        sv_license_key_token: &str,
    ) -> AppResult<()> {
        let ticket_heartbeat = [
            "gameName=gta5&guid=148618792012444134&machineHash=AQAL&machineHashIndex=",
            machine_hash,
            "&server=http%3a%2f%51.91.102.108%3a30120%2f&serverKeyToken=",
            sv_license_key_token,
            "&token=",
            entitlement_id,
            "&i=",
            account_index,
        ]
        .concat();

        let resp = self
            .client
            .post(TICKET_CREATION_URL)
            .body(ticket_heartbeat)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Error inserting bot: {}", e);
                CfxApiError::EntitlementHeartbeatFailed
            })?;

        if !resp.status().is_success() {
            println!("Ticket heartbeat failed: {}", resp.status());
            Err(CfxApiError::StatusCodeNot200)?
        }

        let response_json = resp.json::<serde_json::Value>().await.unwrap();
        if response_json["ticket"].is_null() {
            Err(CfxApiError::TicketResponseNull)?;
        }

        Ok(())
    }

    #[inline(always)]
    pub async fn initialize_player(
        &self,
        machine_hash: &str,
        entitlement_id: &str,
        account_index: &str,
        sv_license_key_token: &str,
    ) -> AppResult<()> {
        self.insert_bot(
            machine_hash,
            entitlement_id,
            account_index,
            sv_license_key_token,
        )
        .await?;

        self.heartbeat(machine_hash, entitlement_id, account_index)
            .await?;

        Ok(())
    }
}
