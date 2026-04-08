//! OpenCoWork Telegram adapter.
//!
//! Provides Telegram bot integration using Teloxide.
//! This is a simplified implementation — production use requires
//! a real Telegram bot token and proper webhook/polling setup.

use serde::{Deserialize, Serialize};

/// Telegram channel identity configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramIdentity {
    /// Bot token from @BotFather.
    pub token: String,
    /// Unique identity ID.
    pub id: String,
    /// Display name.
    pub name: Option<String>,
}

/// Inbound message from Telegram.
#[derive(Debug, Clone, Serialize)]
pub struct TelegramInboundMessage {
    pub channel: String,
    pub identity_id: String,
    pub peer_id: String,
    pub text: String,
    pub from_me: bool,
}

/// Outbound message to Telegram.
#[derive(Debug, Clone, Serialize)]
pub struct TelegramOutboundMessage {
    pub peer_id: String,
    pub text: String,
    pub reply_to: Option<i64>,
}

/// Media types supported by Telegram.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramMediaKind {
    Text,
    Photo,
    Document,
    Voice,
    Video,
    Audio,
}

/// Check if a peer ID looks like a Telegram chat ID.
pub fn is_telegram_peer_id(peer_id: &str) -> bool {
    let trimmed = peer_id.trim();
    trimmed.starts_with('-') && trimmed[1..].chars().all(|c| c.is_ascii_digit())
        || trimmed.chars().all(|c| c.is_ascii_digit())
}

/// Maximum text length for Telegram messages.
pub const MAX_TEXT_LENGTH: usize = 4096;

/// Telegram adapter — simplified stub.
///
/// Full implementation uses Teloxide with webhook or long-polling.
/// This stub provides the types and validation needed by the router.
pub struct TelegramAdapter {
    identity: TelegramIdentity,
}

impl TelegramAdapter {
    pub fn new(identity: TelegramIdentity) -> Self {
        Self { identity }
    }

    /// Get the identity ID.
    pub fn identity_id(&self) -> &str {
        &self.identity.id
    }

    /// Validate the adapter configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.identity.token.is_empty() {
            return Err("Telegram token is required".to_string());
        }
        Ok(())
    }
}
