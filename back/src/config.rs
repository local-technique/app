use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub app_base_url: String,
    pub frontend_origin: String,
    pub frontend_base_url: Option<String>,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub microsoft_client_id: String,
    pub microsoft_client_secret: String,
    pub cookie_key_base64: String,
    pub access_token_jwt_secret: String,
    pub database_url: String,
    pub admin_emails: Option<String>,
    pub listen_addr: Option<String>,
    pub r2_endpoint: String,
    pub r2_attachments_bucket: String,
    pub r2_access_key_id: String,
    pub r2_secret_access_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        let _ = dotenvy::dotenv();
        envy::from_env::<Self>()
    }
}
