mod auth;
mod config;
mod mcp;
mod vault;

use anyhow::Context;
use config::Config;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let cfg = Config::from_env();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", cfg.svc_port))
        .await
        .context("unable to bind to port")?;

    tracing::info!("Aperio starting on port {}", cfg.svc_port);
    tracing::info!("MCP endpoint: http://0.0.0.0:{}/mcp", cfg.svc_port);
    tracing::info!("Health check: http://0.0.0.0:{}/healthz", cfg.svc_port);

    let app = mcp::router(&cfg).await?;

    axum::serve(listener, app)
        .await
        .context("Server has crashed")?;

    Ok(())
}
