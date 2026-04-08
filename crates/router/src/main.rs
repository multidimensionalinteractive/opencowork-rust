//! OpenCoWork Router CLI entry point.

use clap::Parser;
use opencowork_router::{RouterConfig, RouterCore};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "opencowork-router", about = "OpenCoWork message router")]
struct Cli {
    /// Path to router config file (TOML).
    #[arg(short, long, default_value = "router.toml")]
    config: String,

    /// Log level.
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cli.log_level))
        .json()
        .init();

    tracing::info!(config = %cli.config, "starting opencowork-router");

    // Load config (fall back to defaults)
    let config = if std::path::Path::new(&cli.config).exists() {
        let content = std::fs::read_to_string(&cli.config)?;
        toml::from_str::<RouterConfig>(&content)?
    } else {
        tracing::warn!("config file not found, using defaults");
        RouterConfig::default()
    };

    let router = RouterCore::new(config);
    let health = router.health();

    tracing::info!(
        adapters = health.active_adapters,
        "router started"
    );

    // Keep running
    tokio::signal::ctrl_c().await?;
    tracing::info!("shutting down");

    Ok(())
}
