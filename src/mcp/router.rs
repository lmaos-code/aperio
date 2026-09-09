use axum::Router;
use rmcp::transport::{
    StreamableHttpServerConfig,
    streamable_http_server::{session::local::LocalSessionManager, tower::StreamableHttpService},
};

use crate::{
    auth::{AuthState, JwtVerifier, protected_resource_metadata, validate_auth},
    config::Config,
    status::{status_page, status_stats},
    vault::VaultReader,
};

use super::tools::AperioTools;

pub async fn router(cfg: &Config) -> anyhow::Result<Router> {
    let vault = VaultReader::new(std::path::PathBuf::from(&cfg.vault_dir));
    let tools = AperioTools::new(vault);

    let mut server_config = StreamableHttpServerConfig::default();
    server_config.allowed_hosts = cfg.allowed_hosts.clone();

    let mcp_service: StreamableHttpService<AperioTools, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(tools.clone()),
            LocalSessionManager::default().into(),
            server_config,
        );

    let mut protected = Router::new().nest_service("/mcp", mcp_service);

    if cfg.auth_enabled {
        tracing::info!("OIDC authentication enabled");
        let verifier = JwtVerifier::new(cfg).await?;

        let auth_state = AuthState {
            verifier: Some(verifier),
        };
        protected = protected
            .layer(axum::middleware::from_fn(validate_auth))
            .layer(axum::extract::Extension(auth_state));
    } else {
        tracing::warn!("OIDC authentication DISABLED (dev mode)");
    }

    let app = Router::new()
        .route("/healthz", axum::routing::get(healthz))
        .route(
            "/.well-known/oauth-protected-resource",
            axum::routing::get(protected_resource_metadata),
        )
        .route("/status", axum::routing::get(status_page))
        .route("/status/stats", axum::routing::get(status_stats))
        .merge(protected)
        .layer(axum::extract::Extension(cfg.clone()));

    Ok(app)
}

async fn healthz() -> axum::http::StatusCode {
    axum::http::StatusCode::OK
}
