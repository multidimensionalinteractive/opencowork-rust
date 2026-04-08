//! Telegram-specific message types.

use bytes::Bytes;

/// An inbound message received from Telegram.
#[derive(Debug, Clone)]
pub struct InboundMessage {
    /// Platform-specific message ID.
    pub message_id: i64,
    /// Chat this message belongs to.
    pub chat_id: i64,
    /// Sender user ID.
    pub user_id: Option<i64>,
    /// Text content if present.
    pub text: Option<String>,
    /// Reply-to message ID for threading.
    pub reply_to_message_id: Option<i64>,
    /// Media attachments.
    pub media: Vec<InboundMedia>,
}

/// Inbound media attachment types.
#[derive(Debug, Clone)]
pub enum InboundMedia {
    /// Photo with file_id.
    Photo {
        file_id: String,
        caption: Option<String>,
    },
    /// Document with file_id.
    Document {
        file_id: String,
        file_name: Option<String>,
        caption: Option<String>,
    },
    /// Voice message.
    Voice {
        file_id: String,
        duration: Option<u32>,
    },
    /// Video.
    Video {
        file_id: String,
        caption: Option<String>,
    },
}

/// An outbound message to deliver to Telegram.
#[derive(Debug, Clone)]
pub struct OutboundMessage {
    /// Target chat ID.
    pub chat_id: i64,
    /// Text to send (may be chunked).
    pub text: String,
    /// Optional reply-to for threading.
    pub reply_to_message_id: Option<i64>,
    /// Parse mode for formatting.
    pub parse_mode: ParseMode,
}

/// Telegram message parse mode.
#[derive(Debug, Clone, Copy, Default)]
pub enum ParseMode {
    /// No formatting.
    #[default]
    None,
    /// MarkdownV2 formatting.
    MarkdownV2,
    /// HTML formatting.
    Html,
}

impl ParseMode {
    /// Convert to teloxide's ParseMode or None.
    pub fn to_teloxide(&self) -> Option<teloxide::types::ParseMode> {
        match self {
            Self::None => None,
            Self::MarkdownV2 => Some(teloxide::types::ParseMode::MarkdownV2),
            Self::Html => Some(teloxide::types::ParseMode::Html),
        }
    }
}

/// An outbound media message.
#[derive(Debug, Clone)]
pub enum OutboundMedia {
    /// Send a photo by file_id or URL.
    Photo {
        file_id_or_url: String,
        caption: Option<String>,
    },
    /// Send a document by file_id or URL.
    Document {
        file_id_or_url: String,
        caption: Option<String>,
    },
    /// Send raw bytes as a document upload.
    DocumentUpload {
        file_name: String,
        data: Bytes,
        caption: Option<String>,
    },
}
