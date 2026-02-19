use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{warn, trace};

#[derive(Clone)]
pub struct RateLimitEntry {
    pub count: u32,
    pub window_start: Instant,
}

#[derive(Clone)]
pub struct RateLimiter {
    entries: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    max_requests: u32,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_duration_secs: u64) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_duration: Duration::from_secs(window_duration_secs),
        }
    }

    pub async fn check_rate_limit(&self, key: &str) -> Result<bool, String> {
        let mut entries = self.entries.write().await;
        let now = Instant::now();

        if let Some(entry) = entries.get_mut(key) {
            if now.duration_since(entry.window_start) >= self.window_duration {
                entry.count = 1;
                entry.window_start = now;
                Ok(true)
            } else if entry.count < self.max_requests {
                entry.count += 1;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            entries.insert(key.to_string(), RateLimitEntry {
                count: 1,
                window_start: now,
            });
            Ok(true)
        }
    }

    pub async fn cleanup(&self) {
        let mut entries = self.entries.write().await;
        let now = Instant::now();
        entries.retain(|_, entry| {
            now.duration_since(entry.window_start) < self.window_duration * 2
        });
    }
}

#[derive(Clone)]
pub struct RateLimitState {
    pub rate_limiter: RateLimiter,
}

impl RateLimitState {
    pub fn new() -> Self {
        Self {
            rate_limiter: RateLimiter::new(100, 60),
        }
    }
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn extract_client_key(headers: &HeaderMap) -> String {
    if let Some(api_key) = headers.get("X-API-Key").and_then(|h| h.to_str().ok()) {
        format!("api:{}", api_key)
    } else if let Some(auth_header) = headers.get("Authorization").and_then(|h| h.to_str().ok()) {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            format!("jwt:{}", &token[..8.min(token.len())])
        } else {
            "unknown".to_string()
        }
    } else {
        "anonymous".to_string()
    }
}

pub async fn rate_limit_middleware(
    State(state): State<RateLimitState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let client_key = extract_client_key(req.headers());

    match state.rate_limiter.check_rate_limit(&client_key).await {
        Ok(true) => {
            trace!("Rate limit check passed for: {}", client_key);
            Ok(next.run(req).await)
        }
        Ok(false) => {
            warn!("Rate limit exceeded for: {}", client_key);
            Err(StatusCode::TOO_MANY_REQUESTS)
        }
        Err(e) => {
            warn!("Rate limit check error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}