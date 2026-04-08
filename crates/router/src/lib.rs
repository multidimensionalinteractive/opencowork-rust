//! OpenCoWork Router
//!
//! High-performance message routing engine that bridges messaging platforms
//! (Telegram, Slack) with the opencode AI engine. Handles:
//!
//! - Multi-platform adapter management
//! - Message deduplication
//! - Delivery retry with exponential backoff
//! - Error classification and recovery

pub use opencowork_delivery::{DeliveryError, ErrorClass};
pub use opencowork_text::{chunk_text, truncate_text};
pub use opencowork_telegram::{TelegramAdapter, TelegramIdentity, TelegramInboundMessage};
pub use opencowork_slack::{SlackAdapter, SlackIdentity, SlackInboundMessage};

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Router configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Telegram identities to connect.
    pub telegram: Vec<TelegramIdentity>,
    /// Slack identities to connect.
    pub slack: Vec<SlackIdentity>,
    /// Opencode server URL.
    pub opencode_url: String,
    /// Opencode auth token.
    pub opencode_token: Option<String>,
    /// Deduplication window in seconds.
    pub dedup_window_secs: u64,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            telegram: vec![],
            slack: vec![],
            opencode_url: "http://localhost:3000".to_string(),
            opencode_token: None,
            dedup_window_secs: 30,
        }
    }
}

/// Message deduplication store.
pub struct DedupStore {
    seen: DashMap<String, Instant>,
    window: Duration,
}

impl DedupStore {
    pub fn new(window_secs: u64) -> Self {
        Self {
            seen: DashMap::new(),
            window: Duration::from_secs(window_secs),
        }
    }

    /// Check if a message ID has been seen recently.
    /// Returns true if this is a duplicate.
    pub fn is_duplicate(&self, message_id: &str) -> bool {
        // Clean expired entries periodically
        let now = Instant::now();
        self.seen.retain(|_, &mut t| now.duration_since(t) < self.window);

        if self.seen.contains_key(message_id) {
            return true;
        }
        self.seen.insert(message_id.to_string(), now);
        false
    }
}

/// Router health snapshot.
#[derive(Debug, Clone, Serialize)]
pub struct RouterHealth {
    pub uptime_secs: u64,
    pub messages_routed: u64,
    pub messages_dropped: u64,
    pub active_adapters: usize,
    pub dedup_store_size: usize,
}

/// Core router engine.
pub struct RouterCore {
    config: RouterConfig,
    dedup: DedupStore,
    messages_routed: std::sync::atomic::AtomicU64,
    messages_dropped: std::sync::atomic::AtomicU64,
    started_at: Instant,
}

impl RouterCore {
    pub fn new(config: RouterConfig) -> Self {
        let dedup = DedupStore::new(config.dedup_window_secs);
        Self {
            config,
            dedup,
            messages_routed: std::sync::atomic::AtomicU64::new(0),
            messages_dropped: std::sync::atomic::AtomicU64::new(0),
            started_at: Instant::now(),
        }
    }

    /// Process an inbound message. Returns true if routed successfully.
    pub fn route_message(&self, message_id: &str) -> bool {
        if self.dedup.is_duplicate(message_id) {
            self.messages_dropped.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return false;
        }
        self.messages_routed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        true
    }

    /// Get router health.
    pub fn health(&self) -> RouterHealth {
        RouterHealth {
            uptime_secs: self.started_at.elapsed().as_secs(),
            messages_routed: self.messages_routed.load(std::sync::atomic::Ordering::Relaxed),
            messages_dropped: self.messages_dropped.load(std::sync::atomic::Ordering::Relaxed),
            active_adapters: self.config.telegram.len() + self.config.slack.len(),
            dedup_store_size: self.dedup.seen.len(),
        }
    }
}
