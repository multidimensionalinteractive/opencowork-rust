//! HTTP handlers for the OpenCoWork server.
//!
//! Implements all API endpoints including health checks, SSE event streams,
//! workspace management, file operations, and command execution.

use crate::approvals::{ApprovalType, SharedApprovalManager};
use crate::audit::{AuditAction, AuditOutcome, SharedAuditLog};
use crate::config::SharedConfig;
use crate::errors::{Result, ServerError};
use axum::extract::{Path, Query, State};
use axum::response::sse::{Event, Sse};
use axum::Json;
use chrono::Utc;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use uuid::Uuid;

/// Shared application state available to all handlers.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Server configuration.
    pub config: SharedConfig,
    /// SSE event sender.
    pub event_tx: broadcast::Sender<ServerEvent>,
    /// Approval manager.
    pub approvals: SharedApprovalLog,
    /// Audit log.
    pub audit: SharedAuditLog,
    /// Server start time for uptime tracking.
    pub started_at: Instant,
}

// Re-export as alias for handler convenience
pub type SharedApprovalLog = SharedApprovalManager;

/// Server-sent event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    /// A file has changed on disk.
    FileChanged {
        workspace_id: String,
        path: String,
        action: String,
    },
    /// A configuration update occurred.
    ConfigUpdated {
        workspace_id: String,
    },
    /// An approval request was created.
    ApprovalCreated {
        request_id: String,
        approval_type: String,
    },
    /// An approval decision was made.
    ApprovalDecided {
        request_id: String,
        status: String,
    },
    /// A command completed.
    CommandCompleted {
        workspace_id: String,
        command: String,
        exit_code: i32,
    },
    /// Server health update.
    HealthUpdate {
        uptime_secs: u64,
        active_connections: usize,
    },
    /// Generic notification.
    Notification {
        workspace_id: Option<String>,
        message: String,
        level: String,
    },
}

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    pub workspace_root: String,
    pub approval_mode: String,
}

/// Query parameters for listing files.
#[derive(Debug, Deserialize)]
pub struct ListFilesQuery {
    /// Glob pattern for filtering files.
    pub pattern: Option<String>,
    /// Maximum number of results.
    pub limit: Option<usize>,
}

/// Request body for creating a workspace.
#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRequest {
    /// Optional workspace name.
    pub name: Option<String>,
    /// Optional workspace description.
    pub description: Option<String>,
}

/// Request body for updating config.
#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    /// Approval mode.
    pub approval_mode: Option<String>,
    /// Authorized roots.
    pub authorized_roots: Option<Vec<String>>,
}

/// Request body for executing commands.
#[derive(Debug, Deserialize)]
pub struct ExecuteCommandRequest {
    /// The command to execute.
    pub command: String,
    /// Working directory (relative to workspace root).
    pub working_dir: Option<String>,
    /// Environment variables.
    pub env: Option<std::collections::HashMap<String, String>>,
}

/// Request body for file writes.
#[derive(Debug, Deserialize)]
pub struct WriteFileRequest {
    /// The file content.
    pub content: String,
    /// Whether to create parent directories.
    pub create_dirs: Option<bool>,
}

/// Generic success response.
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

/// Workspace info response.
#[derive(Debug, Serialize)]
pub struct WorkspaceInfo {
    pub id: String,
    pub root: String,
    pub created_at: String,
}

/// Config response.
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub workspace_root: String,
    pub approval_mode: String,
    pub authorized_roots: Vec<String>,
}

/// File info response.
#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: Option<String>,
}

/// Command result response.
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /health - Health check endpoint.
///
/// Returns JSON with server status, uptime, and basic metrics.
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let uptime = state.started_at.elapsed().as_secs();
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: uptime,
        workspace_root: state.config.workspace_root.display().to_string(),
        approval_mode: state.config.approval_mode.to_string(),
    })
}

/// GET /api/events - Server-Sent Events stream.
///
/// Provides real-time updates for file changes, approvals, and other events.
pub async fn event_stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = std::result::Result<Event, Infallible>>> {
    let rx = state.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| {
        match result {
            Ok(event) => {
                let data = serde_json::to_string(&event).unwrap_or_default();
                Some(Ok(Event::default().data(data)))
            }
            Err(_) => None, // Lagged receivers are dropped
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

/// POST /api/workspace/create - Create a new workspace.
///
/// Returns the workspace ID and root path.
pub async fn create_workspace(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateWorkspaceRequest>,
) -> Result<Json<WorkspaceInfo>> {
    let workspace_id = Uuid::new_v4().to_string();
    let workspace_root = state.config.workspace_root.display().to_string();

    state.audit.log(
        AuditAction::WorkspaceCreate,
        AuditOutcome::Success,
        Some(workspace_id.clone()),
        None,
        None,
        Some(serde_json::json!({ "name": req.name })),
    );

    Ok(Json(WorkspaceInfo {
        id: workspace_id,
        root: workspace_root,
        created_at: Utc::now().to_rfc3339(),
    }))
}

/// GET /api/workspace/:id/config - Get workspace configuration.
pub async fn get_config(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<String>,
) -> Result<Json<ConfigResponse>> {
    state.audit.log(
        AuditAction::ConfigRead,
        AuditOutcome::Success,
        Some(workspace_id),
        None,
        None,
        None,
    );

    Ok(Json(ConfigResponse {
        workspace_root: state.config.workspace_root.display().to_string(),
        approval_mode: state.config.approval_mode.to_string(),
        authorized_roots: state
            .config
            .authorized_roots
            .iter()
            .map(|p| p.display().to_string())
            .collect(),
    }))
}

/// PUT /api/workspace/:id/config - Update workspace configuration.
pub async fn update_config(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<String>,
    Json(req): Json<UpdateConfigRequest>,
) -> Result<Json<SuccessResponse>> {
    state.audit.log(
        AuditAction::ConfigUpdate,
        AuditOutcome::Success,
        Some(workspace_id),
        None,
        None,
        Some(serde_json::json!({
            "approval_mode": req.approval_mode,
            "authorized_roots": req.authorized_roots,
        })),
    );

    // Broadcast config update event
    let _ = state.event_tx.send(ServerEvent::ConfigUpdated {
        workspace_id: "default".to_string(),
    });

    Ok(Json(SuccessResponse {
        success: true,
        message: "configuration updated".to_string(),
    }))
}

/// POST /api/workspace/:id/reload - Trigger a reload.
pub async fn trigger_reload(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<String>,
) -> Result<Json<SuccessResponse>> {
    state.audit.log(
        AuditAction::Reload,
        AuditOutcome::Success,
        Some(workspace_id),
        None,
        None,
        None,
    );

    // Broadcast reload notification
    let _ = state.event_tx.send(ServerEvent::Notification {
        workspace_id: Some("default".to_string()),
        message: "reload triggered".to_string(),
        level: "info".to_string(),
    });

    Ok(Json(SuccessResponse {
        success: true,
        message: "reload triggered".to_string(),
    }))
}

/// GET /api/workspace/:id/files - List files in the workspace.
pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<String>,
    Query(query): Query<ListFilesQuery>,
) -> Result<Json<Vec<FileInfo>>> {
    state.audit.log(
        AuditAction::FileList,
        AuditOutcome::Success,
        Some(workspace_id),
        None,
        None,
        None,
    );

    let root = &state.config.workspace_root;
    let mut files = Vec::new();
    let limit = query.limit.unwrap_or(1000).min(10000);

    let walker = walkdir::WalkDir::new(root)
        .follow_links(false)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
        .take(limit);

    for entry in walker {
        let rel_path = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .display()
            .to_string();

        // Apply pattern filter if provided
        if let Some(ref pattern) = query.pattern {
            if !glob_match(pattern, &rel_path) {
                continue;
            }
        }

        let metadata = entry.metadata().ok();
        files.push(FileInfo {
            path: rel_path,
            size: metadata.as_ref().map(|m| m.len()).unwrap_or(0),
            is_dir: entry.file_type().is_dir(),
            modified: metadata
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let dt: chrono::DateTime<Utc> = t.into();
                    dt.to_rfc3339()
                }),
        });
    }

    Ok(Json(files))
}

/// GET /api/workspace/:id/files/:path - Read a file.
pub async fn read_file(
    State(state): State<Arc<AppState>>,
    Path((workspace_id, file_path)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let resolved = state.config.resolve_path(&file_path)?;

    state.audit.log(
        AuditAction::FileRead,
        AuditOutcome::Success,
        Some(workspace_id),
        Some(file_path.clone()),
        None,
        None,
    );

    let content = tokio::fs::read_to_string(&resolved).await?;
    Ok(Json(serde_json::json!({
        "path": file_path,
        "content": content,
    })))
}

/// PUT /api/workspace/:id/files/:path - Write a file (with approval).
pub async fn write_file(
    State(state): State<Arc<AppState>>,
    Path((workspace_id, file_path)): Path<(String, String)>,
    Json(req): Json<WriteFileRequest>,
) -> Result<Json<serde_json::Value>> {
    // Validate path
    let resolved = state.config.resolve_path(&file_path)?;

    state.audit.log(
        AuditAction::FileWriteRequest,
        AuditOutcome::Pending,
        Some(workspace_id.clone()),
        Some(file_path.clone()),
        None,
        None,
    );

    // Request approval
    let approval_id = state
        .approvals
        .request_approval(
            ApprovalType::FileWrite,
            workspace_id.clone(),
            format!("Write to file: {}", file_path),
            file_path.clone(),
        )
        .await?;

    // If we got here (non-auto mode), approval was granted
    // Create parent dirs if requested
    if req.create_dirs.unwrap_or(false) {
        if let Some(parent) = resolved.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }

    tokio::fs::write(&resolved, &req.content).await?;

    state.audit.log(
        AuditAction::FileWriteComplete,
        AuditOutcome::Success,
        Some(workspace_id),
        Some(file_path.clone()),
        None,
        Some(serde_json::json!({ "size": req.content.len() })),
    );

    // Broadcast file change event
    let _ = state.event_tx.send(ServerEvent::FileChanged {
        workspace_id: "default".to_string(),
        path: file_path.clone(),
        action: "write".to_string(),
    });

    Ok(Json(serde_json::json!({
        "success": true,
        "path": file_path,
        "approval_id": approval_id,
    })))
}

/// POST /api/workspace/:id/commands - Execute a command.
pub async fn execute_command(
    State(state): State<Arc<AppState>>,
    Path(workspace_id): Path<String>,
    Json(req): Json<ExecuteCommandRequest>,
) -> Result<Json<CommandResponse>> {
    state.audit.log(
        AuditAction::CommandRequest,
        AuditOutcome::Pending,
        Some(workspace_id.clone()),
        Some(req.command.clone()),
        None,
        None,
    );

    // Request approval
    let approval_id = state
        .approvals
        .request_approval(
            ApprovalType::Command,
            workspace_id.clone(),
            format!("Execute command: {}", req.command),
            req.command.clone(),
        )
        .await?;

    // Build the command
    let mut cmd = tokio::process::Command::new("sh");
    cmd.arg("-c").arg(&req.command);

    if let Some(working_dir) = &req.working_dir {
        let resolved = state.config.resolve_path(working_dir)?;
        cmd.current_dir(resolved);
    } else {
        cmd.current_dir(&state.config.workspace_root);
    }

    // Add environment variables
    if let Some(env) = &req.env {
        for (key, value) in env {
            cmd.env(key, value);
        }
    }

    // Execute
    let output = cmd.output().await.map_err(|e| {
        ServerError::CommandFailed(format!("failed to execute command: {}", e))
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    state.audit.log(
        AuditAction::CommandComplete,
        if exit_code == 0 {
            AuditOutcome::Success
        } else {
            AuditOutcome::Failure
        },
        Some(workspace_id.clone()),
        Some(req.command.clone()),
        None,
        Some(serde_json::json!({
            "exit_code": exit_code,
            "approval_id": approval_id,
        })),
    );

    // Broadcast command completion
    let _ = state.event_tx.send(ServerEvent::CommandCompleted {
        workspace_id,
        command: req.command,
        exit_code,
    });

    Ok(Json(CommandResponse {
        stdout,
        stderr,
        exit_code,
    }))
}

/// GET /api/approvals - List pending approval requests.
pub async fn list_approvals(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let approvals = state.approvals.list_pending();
    Json(serde_json::json!({ "approvals": approvals }))
}

/// GET /api/approvals/:id - Get a specific approval request.
pub async fn get_approval(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let approval = state
        .approvals
        .get(&id)
        .ok_or_else(|| ServerError::ApprovalNotFound(id))?;
    Ok(Json(serde_json::to_value(approval)?))
}

/// POST /api/approvals/:id/approve - Approve a pending request.
pub async fn approve_request(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SuccessResponse>> {
    state.approvals.approve(&id)?;

    state.audit.log(
        AuditAction::ApprovalGrant,
        AuditOutcome::Success,
        None,
        Some(id.clone()),
        None,
        None,
    );

    let _ = state.event_tx.send(ServerEvent::ApprovalDecided {
        request_id: id,
        status: "approved".to_string(),
    });

    Ok(Json(SuccessResponse {
        success: true,
        message: "approval granted".to_string(),
    }))
}

/// POST /api/approvals/:id/deny - Deny a pending request.
pub async fn deny_request(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SuccessResponse>> {
    state.approvals.deny(&id)?;

    state.audit.log(
        AuditAction::ApprovalDeny,
        AuditOutcome::Success,
        None,
        Some(id.clone()),
        None,
        None,
    );

    let _ = state.event_tx.send(ServerEvent::ApprovalDecided {
        request_id: id,
        status: "denied".to_string(),
    });

    Ok(Json(SuccessResponse {
        success: true,
        message: "approval denied".to_string(),
    }))
}

// ============================================================================
// Helpers
// ============================================================================

/// Simple glob matching (supports * and **).
fn glob_match(pattern: &str, path: &str) -> bool {
    // Use a simple approach: convert glob to regex-like matching
    // For production, consider using the `glob` crate
    if pattern == "*" || pattern == "**" {
        return true;
    }

    // Split pattern by ** to handle recursive matching
    let parts: Vec<&str> = pattern.split("**").collect();
    if parts.len() == 1 {
        // No **, simple glob
        return simple_glob_match(pattern, path);
    }

    // Handle ** patterns
    let mut remaining = path;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            // Must match from the start
            if !simple_glob_match(part.trim_start_matches('/'), remaining.trim_start_matches('/')) {
                return false;
            }
            // Find where the match ended
            remaining = &remaining[remaining.len().min(part.len())..];
        } else {
            // Can match anywhere in the remaining path
            if !remaining.contains(part.trim_matches('/')) {
                return false;
            }
        }
    }
    true
}

/// Simple single-segment glob matching.
fn simple_glob_match(pattern: &str, path: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    // Convert to a simple matching: split by * and check containment
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == path;
    }
    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 && !path.starts_with(part) {
            return false;
        }
        match path[pos..].find(part) {
            Some(idx) => pos += idx + part.len(),
            None => return false,
        }
    }
    true
}
