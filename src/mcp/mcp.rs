use axum::Router;
use rmcp::transport::{
    StreamableHttpServerConfig,
    streamable_http_server::{session::local::LocalSessionManager, tower::StreamableHttpService},
};

use crate::{config::Config, vault::VaultReader};

use super::tools::AperioTools;

pub fn router(cfg: &Config) -> Router {
    let vault = VaultReader::new(std::path::PathBuf::from(&cfg.vault_dir));
    let tools = AperioTools::new(vault);

    let mcp_service: StreamableHttpService<AperioTools, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(tools.clone()),
            LocalSessionManager::default().into(),
            StreamableHttpServerConfig::default(),
        );

    Router::new()
        .route("/healthz", axum::routing::get(healthz))
        .nest_service("/mcp", mcp_service)
}

async fn healthz() -> axum::http::StatusCode {
    axum::http::StatusCode::OK
}
