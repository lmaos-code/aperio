use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncCheck {
    None,
    File,
    Heartbeat,
    Kubernetes,
}

impl SyncCheck {
    pub fn from_env() -> Self {
        match std::env::var("SYNC_CHECK").as_deref() {
            Ok("file") => Self::File,
            Ok("heartbeat") => Self::Heartbeat,
            Ok("kubernetes") => Self::Kubernetes,
            _ => Self::None,
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub discovery_url: String,
    pub vault_dir: String,
    pub svc_port: String,
    pub auth_enabled: bool,
    pub started_at: Instant,
    pub version: String,
    pub sync_check: SyncCheck,
    pub public_url: Option<String>,
    pub required_audience: Option<String>,
    pub required_issuer: Option<String>,
    pub required_claims: Vec<String>,
    pub mcp_realm: String,
    pub mcp_scopes: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let auth_enabled = std::env::var("OIDC_DISCOVERY_URL").is_ok()
            && std::env::var("LOCAL_AUTH").map_or(true, |v| v != "true");

        Self {
            svc_port: std::env::var("MCP_PORT").unwrap_or_else(|_| "3000".into()),
            vault_dir: std::env::var("OBSIDIAN_VAULT_DIR")
                .unwrap_or_else(|_| "/obsidian-vault".into()),
            discovery_url: std::env::var("OIDC_DISCOVERY_URL").unwrap_or_else(|_| {
                "http://localhost:8081/realms/aperio/.well-known/openid-configuration".into()
            }),
            auth_enabled,
            started_at: Instant::now(),
            version: std::env::var("APERIO_VERSION").unwrap_or_else(|_| "development".into()),
            sync_check: SyncCheck::from_env(),
            public_url: std::env::var("PUBLIC_URL").ok().filter(|s| !s.is_empty()),
            required_audience: std::env::var("OIDC_AUDIENCE")
                .ok()
                .filter(|s| !s.is_empty()),
            required_issuer: std::env::var("OIDC_ISSUER").ok().filter(|s| !s.is_empty()),
            required_claims: std::env::var("OIDC_REQUIRED_CLAIMS")
                .ok()
                .filter(|s| !s.is_empty())
                .map(|s| s.split(',').map(String::from).collect())
                .unwrap_or_default(),
            mcp_realm: std::env::var("MCP_REALM").unwrap_or_else(|_| "mcp".into()),
            mcp_scopes: std::env::var("MCP_SCOPES")
                .ok()
                .filter(|s| !s.is_empty())
                .map_or_else(
                    || vec!["mcp:read".into(), "mcp:write".into()],
                    |s| s.split(',').map(String::from).collect(),
                ),
        }
    }
}
