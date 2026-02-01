//! HTTP server settings

use serde::Deserialize;
use std::time::Duration;

/// HTTP server settings
#[derive(Debug, Clone, Deserialize)]
pub struct HttpConfig {
    #[serde(default = "default_body_limit")]
    #[serde(deserialize_with = "crate::config::de_usize")]
    pub http_body_limit_bytes: usize,

    #[serde(default = "default_request_timeout")]
    #[serde(deserialize_with = "crate::config::de_u64")]
    pub http_request_timeout_seconds: u64,
}

impl HttpConfig {
    /// Get request timeout as Duration
    #[must_use]
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.http_request_timeout_seconds)
    }
}

fn default_body_limit() -> usize {
    1_048_576 // 1MB
}
fn default_request_timeout() -> u64 {
    15
}
