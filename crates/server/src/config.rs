//! Server configuration and CLI argument parsing.
//!
//! Handles workspace resolution, CLI arguments, and configuration merging.

use crate::errors::{Result, ServerError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Approval mode for file write and command operations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalMode {
    /// All operations are automatically approved.
    Auto,
    /// Operations require manual approval via the approval API.
    Manual,
    /// Operations are approved after a timeout if not explicitly denied.
    #[default]
    Timeout,
}

impl std::fmt::Display for ApprovalMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalMode::Auto => write!(f, "auto"),
            ApprovalMode::Manual => write!(f, "manual"),
            ApprovalMode::Timeout => write!(f, "timeout"),
        }
    }
}

impl std::str::FromStr for ApprovalMode {
    type Err = ServerError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(ApprovalMode::Auto),
            "manual" => Ok(ApprovalMode::Manual),
            "timeout" => Ok(ApprovalMode::Timeout),
            _ => Err(ServerError::Config(format!(
                "invalid approval mode '{}': must be auto, manual, or timeout",
                s
            ))),
        }
    }
}

/// Server configuration, built from CLI args and environment.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// The workspace root directory.
    pub workspace_root: PathBuf,

    /// The host to bind to.
    pub host: String,

    /// The port to listen on.
    pub port: u16,

    /// Authentication token (optional).
    pub auth_token: Option<String>,

    /// CORS allowed origins (empty = allow all).
    pub cors_origins: Vec<String>,

    /// Approval mode for mutating operations.
    pub approval_mode: ApprovalMode,

    /// Timeout for approval requests in seconds (when using Timeout mode).
    pub approval_timeout_secs: u64,

    /// Maximum request body size in bytes.
    pub max_body_bytes: usize,

    /// Rate limit: requests per second per IP.
    pub rate_limit_rps: u32,

    /// Authorized filesystem roots (relative to workspace root).
    pub authorized_roots: Vec<PathBuf>,
}

impl ServerConfig {
    /// Create a new server config with sensible defaults.
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            host: "0.0.0.0".to_string(),
            port: 9876,
            auth_token: None,
            cors_origins: vec![],
            approval_mode: ApprovalMode::default(),
            approval_timeout_secs: 30,
            max_body_bytes: 10 * 1024 * 1024, // 10MB
            rate_limit_rps: 100,
            authorized_roots: vec![PathBuf::from(".")],
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<()> {
        if !self.workspace_root.exists() {
            return Err(ServerError::Config(format!(
                "workspace root does not exist: {}",
                self.workspace_root.display()
            )));
        }
        if !self.workspace_root.is_dir() {
            return Err(ServerError::Config(format!(
                "workspace root is not a directory: {}",
                self.workspace_root.display()
            )));
        }
        Ok(())
    }

    /// Resolve a path relative to the workspace root, ensuring it stays within authorized roots.
    pub fn resolve_path(&self, relative_path: &str) -> Result<PathBuf> {
        let clean = relative_path.trim_start_matches('/');
        let resolved = self.workspace_root.join(clean);

        // Canonicalize to prevent path traversal
        let canonical = resolved.canonicalize().map_err(|_| {
            ServerError::FileNotFound(format!("path not found: {}", relative_path))
        })?;
        let root_canonical = self.workspace_root.canonicalize().map_err(|e| {
            ServerError::Internal(format!("cannot canonicalize workspace root: {}", e))
        })?;

        if !canonical.starts_with(&root_canonical) {
            return Err(ServerError::PathTraversalDenied(format!(
                "path '{}' escapes workspace root",
                relative_path
            )));
        }

        // Check against authorized roots
        let is_authorized = self.authorized_roots.iter().any(|root| {
            let root_path = self.workspace_root.join(root);
            if let Ok(root_canon) = root_path.canonicalize() {
                canonical.starts_with(&root_canon)
            } else {
                false
            }
        });

        if !is_authorized {
            return Err(ServerError::PathTraversalDenied(format!(
                "path '{}' is outside authorized roots",
                relative_path
            )));
        }

        Ok(canonical)
    }
}

/// Shared server state, wrapped in Arc for cheap cloning across handlers.
pub type SharedConfig = Arc<ServerConfig>;
