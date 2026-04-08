//! Axum server setup and router construction.
//!
//! Provides the [`Server`] builder for configuring and running the HTTP server
//! with all routes, middleware, and shared state.

use crate::approvals::ApprovalManager;
use crate::audit::AuditLog;
use crate::config::{SharedConfig, ServerConfig};
use crate::handlers;
use crate::middleware::{self, RateLimiter};
use axum::routing::{get, post, put};
use axum::Router;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::net::TcpListener;

/// The OpenCoWork HTTP server.
///
/// Use [`Server::builder`] to create and configure a server instance,
/// then call [`Server::run`] to start listening for connections.
pub struct Server {
    config: SharedConfig,
    router: Router,
    listener: TcpListener,
}

impl Server {
    /// Create a server builder.
    pub fn builder(config: ServerConfig) -> ServerBuilder {
        ServerBuilder { config }
    }

    /// Run the server, accepting connections until cancelled.
    pub async fn run(self) -> anyhow::Result<()> {
        let addr = self.listener.local_addr()?;
        tracing::info!(%addr, "server listening");

        axum::serve(self.listener, self.router).await?;
        Ok(())
    }

    /// Get the local address the server is bound to.
    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }
}

/// Builder for constructing a [`Server`].
pub struct ServerBuilder {
    config: ServerConfig,
}

impl ServerBuilder {
    /// Build the server, binding to the configured address.
    pub async fn build(self) -> anyhow::Result<Server> {
        self.config.validate()?;

        let config = Arc::new(self.config);

        // Create broadcast channels
        let (event_tx, _) = broadcast::channel(1024);
        let (audit_tx, _) = broadcast::channel(4096);

        // Create shared components
        let approvals = Arc::new(ApprovalManager::new(
            config.approval_mode,
            config.approval_timeout_secs,
        ));
        let audit = Arc::new(AuditLog::new(audit_tx));
        let rate_limiter = RateLimiter::new(config.rate_limit_rps);

        // Create application state
        let state = Arc::new(handlers::AppState {
            config: config.clone(),
            event_tx: event_tx.clone(),
            approvals: approvals.clone(),
            audit: audit.clone(),
            started_at: std::time::Instant::now(),
        });

        // Log server start
        audit.log(
            crate::audit::AuditAction::ServerStart,
            crate::audit::AuditOutcome::Success,
            None,
            None,
            None,
            Some(serde_json::json!({
                "host": config.host,
                "port": config.port,
                "workspace_root": config.workspace_root.display().to_string(),
                "approval_mode": config.approval_mode.to_string(),
            })),
        );

        // Build the router
        let router = build_router(state, config.clone(), rate_limiter);

        // Bind the listener
        let addr = format!("{}:{}", config.host, config.port);
        let listener = TcpListener::bind(&addr).await?;
        tracing::info!("bound to {}", addr);

        Ok(Server {
            config,
            router,
            listener,
        })
    }
}

/// Construct the Axum router with all routes and middleware.
fn build_router(
    state: Arc<handlers::AppState>,
    config: SharedConfig,
    rate_limiter: RateLimiter,
) -> Router {
    // API routes
    let api_routes = Router::new()
        // Health
        .route("/health", get(handlers::health_check))
        // Events (SSE)
        .route("/events", get(handlers::event_stream))
        // Workspace management
        .route("/workspace/create", post(handlers::create_workspace))
        .route("/workspace/{id}/config", get(handlers::get_config))
        .route("/workspace/{id}/config", put(handlers::update_config))
        .route("/workspace/{id}/reload", post(handlers::trigger_reload))
        // File operations
        .route("/workspace/{id}/files", get(handlers::list_files))
        .route("/workspace/{id}/files/{*path}", get(handlers::read_file))
        .route("/workspace/{id}/files/{*path}", put(handlers::write_file))
        // Commands
        .route("/workspace/{id}/commands", post(handlers::execute_command))
        // Approvals
        .route("/approvals", get(handlers::list_approvals))
        .route("/approvals/{id}", get(handlers::get_approval))
        .route("/approvals/{id}/approve", post(handlers::approve_request))
        .route("/approvals/{id}/deny", post(handlers::deny_request));

    // Build the full router with middleware
    Router::new()
        .merge(api_routes)
        // Also serve health at root level
        .route("/health", get(handlers::health_check))
        // Apply middleware (innermost first)
        .layer(axum::middleware::from_fn(middleware::logging_middleware))
        .layer(middleware::build_cors_layer(&config))
        // Shared state
        .with_state(state)
        .with_state(rate_limiter)
}
