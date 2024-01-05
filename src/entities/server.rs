#[derive(Debug)]
pub struct Server {
    pub id: String,
    pub name: Option<String>,
    pub cfx_license: String,
    pub cfx_code: Option<String>,
    pub sv_license_key_token: Option<String>,
}
