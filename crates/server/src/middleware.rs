//! HTTP middleware for the Axum server.
//!
//! Provides authentication, CORS, request logging, and rate limiting.

use crate::config::SharedConfig;
use crate::errors::ServerError;
use axum::extract::{Request, State};
use axum::http::{header, HeaderValue, Method};
use axum::middleware::Next;
use axum::response::Response;
use dashmap::DashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;
use tower_http::cors::{AllowOrigin, CorsLayer};

/// Token bucket for rate limiting.
#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
    rate: f64,
    capacity: f64,
}

impl TokenBucket {
    fn new(rate: f64, capacity: f64) -> Self {
        Self {
            tokens: capacity,
            last_refill: Instant::now(),
            rate,
            capacity,
        }
    }

    fn try_consume(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.rate).min(self.capacity);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Rate limiter using per-IP token buckets.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    buckets: Arc<DashMap<IpAddr, TokenBucket>>,
    rate: f64,
    capacity: f64,
}

impl RateLimiter {
    /// Create a new rate limiter with the given requests per second.
    pub fn new(requests_per_second: u32) -> Self {
        let rate = requests_per_second as f64;
        Self {
            buckets: Arc::new(DashMap::new()),
            rate,
            capacity: rate * 2.0, // Allow burst of 2x rate
        }
    }

    /// Check if a request from the given IP is allowed.
    pub fn check(&self, ip: IpAddr) -> bool {
        let mut bucket = self
            .buckets
            .entry(ip)
            .or_insert_with(|| TokenBucket::new(self.rate, self.capacity));
        bucket.try_consume()
    }
}

/// Build a CORS layer from server config.
pub fn build_cors_layer(config: &SharedConfig) -> CorsLayer {
    let mut cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT]);

    if config.cors_origins.is_empty() {
        cors = cors.allow_origin(tower_http::cors::Any);
    } else {
        let origins: Vec<HeaderValue> = config
            .cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        cors = cors.allow_origin(AllowOrigin::list(origins));
    }

    cors
}

/// Authentication middleware.
///
/// Checks the Authorization header against the configured token.
/// If no token is configured, authentication is skipped.
pub async fn auth_middleware(
    State(config): State<SharedConfig>,
    request: Request,
    next: Next,
) -> std::result::Result<Response, ServerError> {
    // Skip auth if no token configured
    let expected_token = match &config.auth_token {
        Some(token) => token,
        None => return Ok(next.run(request).await),
    };

    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(header_value) => {
            let token = header_value.strip_prefix("Bearer ").unwrap_or(header_value);
            if token == expected_token {
                Ok(next.run(request).await)
            } else {
                Err(ServerError::Unauthorized("invalid token".to_string()))
            }
        }
        None => Err(ServerError::Unauthorized(
            "missing authorization header".to_string(),
        )),
    }
}

/// Request logging middleware.
///
/// Logs request method, path, status, and duration using tracing.
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    tracing::info!(
        %method,
        %uri,
        %status,
        duration_ms = duration.as_millis() as u64,
        "request"
    );

    response
}

/// Rate limiting middleware.
pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    request: Request,
    next: Next,
) -> std::result::Result<Response, ServerError> {
    // Extract IP from the request
    // In production, this would use a proper extraction method
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(IpAddr::from([127, 0, 0, 1]));

    if limiter.check(ip) {
        Ok(next.run(request).await)
    } else {
        Err(ServerError::RateLimited)
    }
}
