//! Approval system for mutating operations.
//!
//! Manages approval requests for file writes and command execution.
//! Supports three modes: auto (always approved), manual (explicit approval),
//! and timeout (auto-approve after delay).

use crate::config::ApprovalMode;
use crate::errors::{Result, ServerError};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use uuid::Uuid;

/// The current status of an approval request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    /// Waiting for approval decision.
    Pending,
    /// Operation was approved.
    Approved,
    /// Operation was denied.
    Denied,
    /// Operation expired without a decision.
    Expired,
}

/// The type of operation needing approval.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalType {
    /// Writing to a file.
    FileWrite,
    /// Executing a command.
    Command,
}

/// An approval request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique identifier.
    pub id: String,
    /// When the request was created.
    pub created_at: DateTime<Utc>,
    /// When the request expires (if using timeout mode).
    pub expires_at: Option<DateTime<Utc>>,
    /// The type of operation.
    pub approval_type: ApprovalType,
    /// The workspace ID.
    pub workspace_id: String,
    /// Description of the operation.
    pub description: String,
    /// The resource path (for file writes) or command (for commands).
    pub resource: String,
    /// Current status.
    pub status: ApprovalStatus,
}

/// Manages approval requests.
///
/// Uses a concurrent map for lock-free access from multiple handlers.
/// The sender half of the oneshot channel is stored separately to avoid
/// Clone requirements on the DashMap value type.
pub struct ApprovalManager {
    /// Active approval request metadata (serializable).
    requests: Arc<DashMap<String, ApprovalRequest>>,
    /// Pending oneshot senders keyed by request ID.
    senders: Arc<DashMap<String, oneshot::Sender<ApprovalStatus>>>,
    /// The approval mode.
    mode: ApprovalMode,
    /// Default timeout duration for timeout mode.
    timeout: Duration,
}

impl std::fmt::Debug for ApprovalManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApprovalManager")
            .field("pending_count", &self.requests.len())
            .field("mode", &self.mode)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl ApprovalManager {
    /// Create a new approval manager.
    pub fn new(mode: ApprovalMode, timeout_secs: u64) -> Self {
        Self {
            requests: Arc::new(DashMap::new()),
            senders: Arc::new(DashMap::new()),
            mode,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Request approval for an operation. Returns a future that resolves
    /// when the approval is granted, denied, or expires.
    pub async fn request_approval(
        &self,
        approval_type: ApprovalType,
        workspace_id: String,
        description: String,
        resource: String,
    ) -> Result<String> {
        match self.mode {
            ApprovalMode::Auto => {
                tracing::debug!("auto-approving operation: {}", description);
                return Ok(Uuid::new_v4().to_string());
            }
            ApprovalMode::Manual | ApprovalMode::Timeout => {}
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = if self.mode == ApprovalMode::Timeout {
            Some(now + chrono::Duration::from_std(self.timeout).unwrap_or(chrono::Duration::seconds(30)))
        } else {
            None
        };

        let request = ApprovalRequest {
            id: id.clone(),
            created_at: now,
            expires_at,
            approval_type,
            workspace_id,
            description,
            resource,
            status: ApprovalStatus::Pending,
        };

        let (tx, rx) = oneshot::channel();

        self.requests.insert(id.clone(), request);
        self.senders.insert(id.clone(), tx);

        tracing::info!(id = %id, "approval request created");

        // If timeout mode, spawn a task to auto-expire
        if self.mode == ApprovalMode::Timeout {
            let requests = Arc::clone(&self.requests);
            let senders = Arc::clone(&self.senders);
            let timeout_id = id.clone();
            let timeout_dur = self.timeout;
            tokio::spawn(async move {
                tokio::time::sleep(timeout_dur).await;
                if let Some((_, sender)) = senders.remove(&timeout_id) {
                    tracing::info!(id = %timeout_id, "approval expired");
                    let _ = sender.send(ApprovalStatus::Expired);
                }
                // Also remove the request metadata
                requests.remove(&timeout_id);
            });
        }

        // Wait for the approval decision
        match rx.await {
            Ok(status) => {
                self.requests.remove(&id);
                self.senders.remove(&id);
                match status {
                    ApprovalStatus::Approved => Ok(id),
                    ApprovalStatus::Denied => Err(ServerError::ApprovalNotFound(
                        "operation was denied".to_string(),
                    )),
                    ApprovalStatus::Expired => Err(ServerError::ApprovalTimeout(
                        "approval request timed out".to_string(),
                    )),
                    ApprovalStatus::Pending => Err(ServerError::Internal(
                        "unexpected pending status".to_string(),
                    )),
                }
            }
            Err(_) => {
                self.requests.remove(&id);
                self.senders.remove(&id);
                Err(ServerError::ApprovalNotFound(
                    "approval channel closed".to_string(),
                ))
            }
        }
    }

    /// Approve a pending request.
    pub fn approve(&self, id: &str) -> Result<()> {
        let (_, sender) = self
            .senders
            .remove(id)
            .ok_or_else(|| ServerError::ApprovalNotFound(id.to_string()))?;

        tracing::info!(id = %id, "approval granted");
        let _ = sender.send(ApprovalStatus::Approved);
        Ok(())
    }

    /// Deny a pending request.
    pub fn deny(&self, id: &str) -> Result<()> {
        let (_, sender) = self
            .senders
            .remove(id)
            .ok_or_else(|| ServerError::ApprovalNotFound(id.to_string()))?;

        tracing::info!(id = %id, "approval denied");
        let _ = sender.send(ApprovalStatus::Denied);
        Ok(())
    }

    /// List all pending approval requests.
    pub fn list_pending(&self) -> Vec<ApprovalRequest> {
        self.requests
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get a specific approval request.
    pub fn get(&self, id: &str) -> Option<ApprovalRequest> {
        self.requests.get(id).map(|entry| entry.value().clone())
    }

    /// Get the approval mode.
    pub fn mode(&self) -> ApprovalMode {
        self.mode
    }
}

/// Shared approval manager type.
pub type SharedApprovalManager = Arc<ApprovalManager>;
