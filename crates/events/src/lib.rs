//! OpenCoWork events crate.
//!
//! Provides event types and broadcasting infrastructure.

/// Placeholder event type.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub kind: String,
    pub data: serde_json::Value,
}
