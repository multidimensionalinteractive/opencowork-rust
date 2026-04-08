//! OpenCoWork configuration crate.
//!
//! Provides configuration types and parsing for the OpenCoWork platform.

/// Placeholder for workspace configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceConfig {
    pub name: String,
    pub root: String,
}
