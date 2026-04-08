//! Exponential backoff retry logic with jitter.

use std::time::Duration;

use rand::Rng;
use tracing::{debug, warn};

use crate::DeliveryError;

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of attempts (including the first).
    pub max_attempts: u32,
    /// Base delay in milliseconds.
    pub base_delay_ms: u64,
    /// Maximum delay cap in milliseconds.
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            base_delay_ms: 100,
            max_delay_ms: 30_000,
        }
    }
}

impl RetryConfig {
    /// Create a new config with the given max attempts and base delay.
    pub fn new(max_attempts: u32, base_delay_ms: u64) -> Self {
        Self {
            max_attempts,
            base_delay_ms,
            max_delay_ms: 30_000,
        }
    }

    /// Calculate delay for a given attempt number (0-indexed) with jitter.
    ///
    /// Uses `base * 2^attempt` with full jitter: `rand(0, calculated)`.
    fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let exp = attempt.min(20); // prevent overflow
        let base = self.base_delay_ms.saturating_mul(1u64 << exp);
        let capped = base.min(self.max_delay_ms);

        let mut rng = rand::thread_rng();
        let jittered = rng.gen_range(0..=capped);
        Duration::from_millis(jittered)
    }
}

/// Execute an async operation with exponential backoff retry.
///
/// Retries on [`DeliveryError::Transient`] and [`DeliveryError::RateLimited`].
/// Returns immediately on success or [`DeliveryError::Permanent`].
pub async fn with_delivery_retry<F, Fut, T>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, DeliveryError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, DeliveryError>>,
{
    let mut last_err = None;

    for attempt in 0..config.max_attempts {
        match operation().await {
            Ok(val) => return Ok(val),
            Err(err) => {
                if !err.is_retryable() {
                    debug!(attempt, error = %err, "permanent error, not retrying");
                    return Err(err);
                }

                let delay = if let DeliveryError::RateLimited { retry_after_ms } = &err {
                    Duration::from_millis(*retry_after_ms)
                } else {
                    config.delay_for_attempt(attempt)
                };

                if attempt + 1 < config.max_attempts {
                    warn!(
                        attempt = attempt + 1,
                        max_attempts = config.max_attempts,
                        delay_ms = delay.as_millis() as u64,
                        error = %err,
                        "retrying after error"
                    );
                    tokio::time::sleep(delay).await;
                }

                last_err = Some(err);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| DeliveryError::Transient("max retries exceeded".into())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_succeeds_on_third_attempt() {
        let config = RetryConfig::new(5, 10);
        let mut count = 0u32;

        let result = with_delivery_retry(&config, || {
            count += 1;
            async move {
                if count < 3 {
                    Err(DeliveryError::Transient("fail".into()))
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_retry_stops_on_permanent_error() {
        let config = RetryConfig::new(5, 10);
        let mut count = 0u32;

        let result: Result<(), _> = with_delivery_retry(&config, || {
            count += 1;
            async move { Err(DeliveryError::Permanent("bad".into())) }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(count, 1); // no retries
    }

    #[tokio::test]
    async fn test_retry_exhausts_attempts() {
        let config = RetryConfig::new(3, 10);
        let mut count = 0u32;

        let result: Result<(), _> = with_delivery_retry(&config, || {
            count += 1;
            async move { Err(DeliveryError::Transient("fail".into())) }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(count, 3);
    }
}
