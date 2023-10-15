use serde::Deserialize;
use sqlx::FromRow;

#[derive(Debug, Deserialize, FromRow)]
pub struct Server {
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "cfxLicense", skip_deserializing)]
    pub cfx_license: Option<String>,
    #[serde(rename = "cfxCode", skip_deserializing)]
    pub cfx_code: Option<String>,
    #[serde(rename = "sv_licenseKeyToken")]
    pub sv_license_key_token: Option<String>,
}
