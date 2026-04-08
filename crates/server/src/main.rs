//! OpenCoWork Server CLI entry point.
//!
//! Parses CLI arguments and starts the Axum HTTP server.

use clap::Parser;
use opencowork_server::config::{ApprovalMode, ServerConfig};
use opencowork_server::server::Server;
use std::path::PathBuf;
use tracing_subscriber::{fmt, EnvFilter};

/// OpenCoWork high-performance server.
///
/// A Rust replacement for the TypeScript/Bun OpenWork server,
/// built with Axum for maximum throughput and minimal latency.
#[derive(Debug, Parser)]
#[command(name = "opencowork-server", version, about)]
struct Cli {
    /// Workspace root directory.
    #[arg(short, long, env = "OPENCOWORK_WORKSPACE", default_value = ".")]
    workspace: PathBuf,

    /// Port to listen on.
    #[arg(short, long, env = "OPENCOWORK_PORT", default_value = "9876")]
    port: u16,

    /// Host to bind to.
    #[arg(long, env = "OPENCOWORK_HOST", default_value = "0.0.0.0")]
    host: String,

    /// Authentication token (optional).
    #[arg(short, long, env = "OPENCOWORK_TOKEN")]
    token: Option<String>,

    /// CORS allowed origins (comma-separated, empty = allow all).
    #[arg(long, env = "OPENCOWORK_CORS")]
    cors: Option<String>,

    /// Approval mode: auto, manual, or timeout.
    #[arg(short, long, env = "OPENCOWORK_APPROVAL_MODE", default_value = "timeout")]
    approval: ApprovalMode,

    /// Approval timeout in seconds (for timeout mode).
    #[arg(long, env = "OPENCOWORK_APPROVAL_TIMEOUT", default_value = "30")]
    approval_timeout: u64,

    /// Rate limit (requests per second per IP).
    #[arg(long, env = "OPENCOWORK_RATE_LIMIT", default_value = "100")]
    rate_limit: u32,

    /// Log level (trace, debug, info, warn, error).
    #[arg(long, env = "RUST_LOG", default_value = "info")]
    log_level: String,

    /// JSON log output format.
    #[arg(long, env = "OPENCOWORK_JSON_LOGS")]
    json_logs: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&cli.log_level));

    if cli.json_logs {
        fmt()
            .json()
            .with_env_filter(filter)
            .with_target(true)
            .with_thread_ids(true)
            .init();
    } else {
        fmt()
            .with_env_filter(filter)
            .with_target(true)
            .init();
    }

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        workspace = %cli.workspace.display(),
        port = cli.port,
        host = %cli.host,
        approval_mode = %cli.approval,
        "starting opencowork-server"
    );

    // Build server config
    let mut config = ServerConfig::new(cli.workspace);
    config.host = cli.host;
    config.port = cli.port;
    config.auth_token = cli.token;
    config.approval_mode = cli.approval;
    config.approval_timeout_secs = cli.approval_timeout;
    config.rate_limit_rps = cli.rate_limit;

    if let Some(cors) = cli.cors {
        config.cors_origins = cors
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    // Build and run the server
    let server = Server::builder(config).build().await?;
    server.run().await?;

    Ok(())
}
