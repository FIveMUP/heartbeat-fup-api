use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Server {
    pub id: Option<String>,
    pub name: Option<String>,
    pub cfxLicense: Option<String>,
    pub cfxCode: Option<String>,
    pub sv_licenseKeyToken: Option<String>,
}
