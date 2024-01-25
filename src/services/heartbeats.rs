use crate::{
    error::{AppResult, CfxApiError},
    structs::client_manager::ClientManager,
};
use deadpool::{managed::Pool, Status};

const TICKET_CREATION_URL: &str = "https://lambda.fivem.net/api/ticket/create";
const HEARTBEAT_URL: &str = "https://cnl-hb-live.fivem.net/api/validate/entitlement";

#[derive(Clone)]
pub struct FivemService {
    clients: Pool<ClientManager>,
}

impl FivemService {
    pub fn new() -> Self {
        let mgr = ClientManager;
        let pool = Pool::builder(mgr).max_size(5000).build().unwrap();

        Self { clients: pool }
    }

    #[inline]
    pub fn status(&self) -> Status {
        self.clients.status()
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

            let client = self.clients.get().await.unwrap();

            client
                .execute_post(HEARTBEAT_URL, entitlement_heartbeat)
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

        let client = self.clients.get().await.unwrap();

        let resp = client
            .execute_post(TICKET_CREATION_URL, ticket_heartbeat)
            .await
            .map_err(|_| CfxApiError::TicketHeartbeatFailed)?;

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
