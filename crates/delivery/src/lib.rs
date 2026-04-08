//! OpenCoWork delivery crate.
//!
//! Provides retry logic with exponential backoff and jitter for
//! reliable message delivery across platforms.

mod retry;

pub use retry::{with_delivery_retry, RetryConfig};

use std::fmt;

/// Classification of delivery errors for observability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    /// Transient failure — retry may succeed (network timeout, 5xx).
    Transient,
    /// Permanent failure — retry will not help (auth error, 4xx).
    Permanent,
    /// Rate-limited — retry after the suggested delay.
    RateLimited,
}

/// Delivery errors with classification for retry decisions.
#[derive(Debug, thiserror::Error)]
pub enum DeliveryError {
    /// A transient network or server error.
    #[error("transient delivery error: {0}")]
    Transient(String),

    /// A permanent client error (bad request, auth failure).
    #[error("permanent delivery error: {0}")]
    Permanent(String),

    /// Rate limited by the upstream service.
    #[error("rate limited, retry after {retry_after_ms}ms")]
    RateLimited {
        /// Suggested milliseconds to wait before retry.
        retry_after_ms: u64,
    },
}

impl DeliveryError {
    /// Classify this error for retry policy decisions.
    pub fn classify(&self) -> ErrorClass {
        match self {
            Self::Transient(_) => ErrorClass::Transient,
            Self::Permanent(_) => ErrorClass::Permanent,
            Self::RateLimited { .. } => ErrorClass::RateLimited,
        }
    }

    /// Whether this error is retryable under normal circumstances.
    pub fn is_retryable(&self) -> bool {
        !matches!(self.classify(), ErrorClass::Permanent)
    }
}

impl fmt::Display for ErrorClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transient => write!(f, "transient"),
            Self::Permanent => write!(f, "permanent"),
            Self::RateLimited => write!(f, "rate_limited"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_classification() {
        let t = DeliveryError::Transient("timeout".into());
        assert!(t.is_retryable());
        assert_eq!(t.classify(), ErrorClass::Transient);

        let p = DeliveryError::Permanent("bad auth".into());
        assert!(!p.is_retryable());
        assert_eq!(p.classify(), ErrorClass::Permanent);

        let r = DeliveryError::RateLimited { retry_after_ms: 1000 };
        assert!(r.is_retryable());
        assert_eq!(r.classify(), ErrorClass::RateLimited);
    }
}
