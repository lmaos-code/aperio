use axum::Router;
use rmcp::transport::{
    StreamableHttpServerConfig,
    streamable_http_server::{session::local::LocalSessionManager, tower::StreamableHttpService},
};

use crate::{
    auth::{AuthState, JwtVerifier, protected_resource_metadata, validate_auth},
    config::Config,
    vault::VaultReader,
};

use super::tools::AperioTools;

pub async fn router(cfg: &Config) -> anyhow::Result<Router> {
    let vault = VaultReader::new(std::path::PathBuf::from(&cfg.vault_dir));
    let tools = AperioTools::new(vault);

    let mcp_service: StreamableHttpService<AperioTools, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(tools.clone()),
            LocalSessionManager::default().into(),
            StreamableHttpServerConfig::default(),
        );

    let mut mcp_router = Router::new().nest_service("/mcp", mcp_service);

    if cfg.auth_enabled {
        tracing::info!("OIDC authentication enabled");
        let verifier = JwtVerifier::new(cfg).await?;

        let resource_url = format!("http://localhost:{}", cfg.svc_port);
        let auth_server_url = cfg
            .issuer_url
            .trim_end_matches('/')
            .trim_end_matches("/.well-known/openid-configuration")
            .to_string();

        let auth_state = AuthState {
            verifier: Some(verifier),
            resource_url,
            auth_server_url,
        };
        mcp_router = mcp_router.layer(axum::middleware::from_fn_with_state(
            auth_state,
            validate_auth,
        ));
    } else {
        tracing::warn!("OIDC authentication DISABLED (dev mode)");
    }

    let well_known = Router::new().route(
        "/.well-known/oauth-protected-resource",
        axum::routing::get(protected_resource_metadata),
    );

    let app = Router::new()
        .route("/healthz", axum::routing::get(healthz))
        .merge(well_known)
        .merge(mcp_router)
        .with_state(cfg.clone());

    Ok(app)
}

async fn healthz() -> axum::http::StatusCode {
    axum::http::StatusCode::OK
}
