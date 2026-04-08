//! OpenCoWork Server Library
//!
//! High-performance Rust server for OpenCoWork, replacing the TypeScript/Bun server
//! with an Axum-based implementation. Provides:
//!
//! - **SSE event streaming** for real-time updates
//! - **File operations** scoped to authorized roots
//! - **Command execution** with approval workflows
//! - **Audit logging** for compliance
//! - **Rate limiting** and authentication middleware
//!
//! # Architecture
//!
//! The server uses:
//! - `tokio::sync::broadcast` for SSE event fan-out
//! - `dashmap::DashMap` for concurrent workspace state
//! - `arc-swap` for lock-free configuration updates
//! - `tower`/`tower-http` for middleware composition
//!
//! # Example
//!
//! ```rust,no_run
//! use opencowork_server::server::Server;
//! use opencowork_server::config::{ServerConfig, ApprovalMode};
//! use std::path::PathBuf;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = ServerConfig {
//!     workspace_root: PathBuf::from("./workspace"),
//!     host: "0.0.0.0".to_string(),
//!     port: 9876,
//!     approval_mode: ApprovalMode::Auto,
//!     ..ServerConfig::new(PathBuf::from("./workspace"))
//! };
//!
//! let server = Server::builder(config).build().await?;
//! server.run().await?;
//! # Ok(())
//! # }
//! ```

pub mod approvals;
pub mod audit;
pub mod config;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod server;

// Re-export key types for convenience
pub use approvals::{ApprovalManager, ApprovalRequest, ApprovalStatus, ApprovalType};
pub use audit::{AuditAction, AuditEntry, AuditLog, AuditOutcome};
pub use config::{ApprovalMode, ServerConfig, SharedConfig};
pub use errors::{Result, ServerError};
pub use handlers::AppState;
pub use middleware::RateLimiter;
pub use server::Server;
