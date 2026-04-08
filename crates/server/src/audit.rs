//! Audit trail logging.
//!
//! Records all significant operations for compliance and debugging.
//! Each entry captures who did what, when, and the outcome.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

/// The type of operation being audited.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    /// Server started.
    ServerStart,
    /// Server stopped.
    ServerStop,
    /// Workspace created.
    WorkspaceCreate,
    /// Configuration read.
    ConfigRead,
    /// Configuration updated.
    ConfigUpdate,
    /// Reload triggered.
    Reload,
    /// File listed.
    FileList,
    /// File read.
    FileRead,
    /// File write requested.
    FileWriteRequest,
    /// File write completed.
    FileWriteComplete,
    /// Command execution requested.
    CommandRequest,
    /// Command execution completed.
    CommandComplete,
    /// Approval granted.
    ApprovalGrant,
    /// Approval denied.
    ApprovalDeny,
    /// Approval expired.
    ApprovalExpire,
}

/// Outcome of an audited operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    /// Operation succeeded.
    Success,
    /// Operation failed.
    Failure,
    /// Operation is pending approval.
    Pending,
}

/// A single audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique identifier for this audit entry.
    pub id: String,
    /// When the operation occurred.
    pub timestamp: DateTime<Utc>,
    /// The workspace involved, if any.
    pub workspace_id: Option<String>,
    /// The action performed.
    pub action: AuditAction,
    /// The outcome of the action.
    pub outcome: AuditOutcome,
    /// The path or resource involved.
    pub resource: Option<String>,
    /// IP address of the requester.
    pub remote_addr: Option<String>,
    /// Additional details.
    pub details: Option<serde_json::Value>,
}

/// Thread-safe audit logger.
///
/// Maintains an in-memory ring buffer of recent audit entries
/// and broadcasts new entries for SSE consumers.
#[derive(Debug, Clone)]
pub struct AuditLog {
    sender: broadcast::Sender<AuditEntry>,
}

impl AuditLog {
    /// Create a new audit log with the given broadcast channel.
    pub fn new(sender: broadcast::Sender<AuditEntry>) -> Self {
        Self { sender }
    }

    /// Record an audit entry.
    pub fn record(&self, entry: AuditEntry) {
        tracing::info!(
            id = %entry.id,
            action = ?entry.action,
            outcome = ?entry.outcome,
            resource = ?entry.resource,
            "audit"
        );
        // Ignore if no receivers - that's fine
        let _ = self.sender.send(entry);
    }

    /// Create and record an entry in one call.
    pub fn log(
        &self,
        action: AuditAction,
        outcome: AuditOutcome,
        workspace_id: Option<String>,
        resource: Option<String>,
        remote_addr: Option<String>,
        details: Option<serde_json::Value>,
    ) {
        let entry = AuditEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            workspace_id,
            action,
            outcome,
            resource,
            remote_addr,
            details,
        };
        self.record(entry);
    }

    /// Subscribe to audit events.
    pub fn subscribe(&self) -> broadcast::Receiver<AuditEntry> {
        self.sender.subscribe()
    }
}

/// Shared audit log type.
pub type SharedAuditLog = Arc<AuditLog>;
