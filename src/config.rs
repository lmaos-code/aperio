use std::time::Instant;

#[derive(Clone)]
pub struct Config {
    pub issuer_url: String,
    pub vault_dir: String,
    pub svc_port: String,
    pub auth_enabled: bool,
    pub started_at: Instant,
    pub version: String,
}

impl Config {
    pub fn from_env() -> Self {
        let auth_enabled = std::env::var("OIDC_ISSUER_URL").is_ok()
            && std::env::var("LOCAL_AUTH").map_or(true, |v| v != "true");

        Self {
            svc_port: std::env::var("MCP_PORT").unwrap_or_else(|_| "3000".into()),
            vault_dir: std::env::var("OBSIDIAN_VAULT_DIR")
                .unwrap_or_else(|_| "/obsidian-vault".into()),
            issuer_url: std::env::var("OIDC_ISSUER_URL").unwrap_or_else(|_| {
                "http://localhost:8081/realms/aperio/.well-known/openid-configuration".into()
            }),
            auth_enabled,
            started_at: Instant::now(),
            version: std::env::var("APERIO_VERSION").unwrap_or_else(|_| "development".into()),
        }
    }
}
