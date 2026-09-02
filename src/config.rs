#[derive(Clone)]
pub struct Config {
    pub issuer_url: String,
    pub vault_dir: String,
    pub svc_port: String,
    pub oidc_enabled: bool,
    pub local_auth_enabled: bool,

}

impl Config {
    pub fn from_env() -> Self {
        Self {
            svc_port: std::env::var("MCP_PORT").unwrap_or_else(|_| "3000".into()),
            vault_dir: std::env::var("OBSIDIAN_VAULT_DIR").unwrap_or_else(|_| "/obsidian-vault".into()),
            issuer_url: std::env::var("OIDC_ISSUER_URL")
                .unwrap_or_else(|_| "http://localhost:8081/realms/aperio/.well-known/openid-configuration".into()),
            oidc_enabled:  std::env::var("OIDC_ISSUER_URL").is_ok(),
            local_auth_enabled: std::env::var("LOCAL_AUTH").is_ok_and(|val| val.parse::<bool>().unwrap_or(false)),
        }
    }
} 

