//! Error types for the OpenCoWork server.
//!
//! Provides a unified error type with Axum `IntoResponse` implementation
//! for consistent HTTP error responses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::io;

/// The main error type for server operations.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    /// The requested workspace was not found.
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(String),

    /// The requested file was not found.
    #[error("file not found: {0}")]
    FileNotFound(String),

    /// Path traversal attempt detected.
    #[error("path traversal denied: {0}")]
    PathTraversalDenied(String),

    /// The requested operation requires approval.
    #[error("operation requires approval: {0}")]
    ApprovalRequired(String),

    /// The approval request was not found or expired.
    #[error("approval not found or expired: {0}")]
    ApprovalNotFound(String),

    /// The approval request timed out.
    #[error("approval timed out: {0}")]
    ApprovalTimeout(String),

    /// Authentication failed.
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// Rate limit exceeded.
    #[error("rate limit exceeded")]
    RateLimited,

    /// Invalid request body or parameters.
    #[error("bad request: {0}")]
    BadRequest(String),

    /// Command execution failed.
    #[error("command execution failed: {0}")]
    CommandFailed(String),

    /// IO error from filesystem operations.
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    /// JSON serialization/deserialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Configuration error.
    #[error("config error: {0}")]
    Config(String),

    /// Internal server error.
    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            ServerError::WorkspaceNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ServerError::FileNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ServerError::PathTraversalDenied(_) => (StatusCode::FORBIDDEN, self.to_string()),
            ServerError::ApprovalRequired(_) => (StatusCode::ACCEPTED, self.to_string()),
            ServerError::ApprovalNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ServerError::ApprovalTimeout(_) => (StatusCode::REQUEST_TIMEOUT, self.to_string()),
            ServerError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            ServerError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            ServerError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServerError::CommandFailed(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            ServerError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string()),
            ServerError::Json(_) => (StatusCode::BAD_REQUEST, "invalid json".to_string()),
            ServerError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string()),
            ServerError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string()),
        };

        tracing::error!(%status, error = %self, "request error");

        let body = json!({
            "error": error_message,
            "status": status.as_u16(),
        });

        (status, Json(body)).into_response()
    }
}

/// Result type alias using [`ServerError`].
pub type Result<T> = std::result::Result<T, ServerError>;
