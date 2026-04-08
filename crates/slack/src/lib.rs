//! OpenCoWork Slack adapter.
//!
//! Provides Slack integration via Socket Mode or Events API.
//! This is a simplified implementation — production use requires
//! a Slack app with proper tokens and event subscriptions.

use serde::{Deserialize, Serialize};

/// Slack workspace identity configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackIdentity {
    /// Slack Bot Token (xoxb-...)
    pub bot_token: String,
    /// App-level Token for Socket Mode (xapp-...)
    pub app_token: String,
    /// Unique identity ID.
    pub id: String,
    /// Workspace name.
    pub workspace: Option<String>,
}

/// Inbound message from Slack.
#[derive(Debug, Clone, Serialize)]
pub struct SlackInboundMessage {
    pub channel: String,
    pub identity_id: String,
    pub peer_id: String,
    pub text: String,
    pub thread_ts: Option<String>,
    pub from_me: bool,
}

/// Outbound message to Slack.
#[derive(Debug, Clone, Serialize)]
pub struct SlackOutboundMessage {
    pub peer_id: String,
    pub text: String,
    pub thread_ts: Option<String>,
}

/// Maximum text length for Slack messages.
pub const MAX_TEXT_LENGTH: usize = 40_000;

/// Slack adapter — simplified stub.
///
/// Full implementation uses slack-morphism with Socket Mode.
/// This stub provides the types needed by the router.
pub struct SlackAdapter {
    identity: SlackIdentity,
}

impl SlackAdapter {
    pub fn new(identity: SlackIdentity) -> Self {
        Self { identity }
    }

    /// Get the identity ID.
    pub fn identity_id(&self) -> &str {
        &self.identity.id
    }

    /// Validate the adapter configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.identity.bot_token.is_empty() {
            return Err("Slack bot token is required".to_string());
        }
        if self.identity.app_token.is_empty() {
            return Err("Slack app token is required".to_string());
        }
        Ok(())
    }
}
