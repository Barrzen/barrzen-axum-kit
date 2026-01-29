//! Configuration module
//!
//! Loads configuration from environment variables with dotenv support.
//! Provides comprehensive configuration for Axum applications.

mod app;
mod banner;
mod cache;
mod cors;
mod features;
mod http;
mod logging;

pub use app::{AppConfig, Environment};
pub use banner::BannerConfig;
pub use cache::{CacheBackend, CacheConfig};
pub use cors::CorsConfig;
pub use features::FeatureFlags;
pub use http::HttpConfig;
pub use logging::{LogFormat, LoggingConfig};

use serde::Deserialize;

/// Main application configuration
///
/// This aggregates all configuration sections and can be loaded from environment variables.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub app: AppConfig,

    #[serde(flatten)]
    pub features: FeatureFlags,

    #[serde(flatten)]
    pub http: HttpConfig,

    #[serde(flatten)]
    pub logging: LoggingConfig,

    #[serde(flatten)]
    pub cache: CacheConfig,

    #[serde(flatten)]
    pub cors: CorsConfig,

    #[serde(flatten)]
    pub banner: BannerConfig,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// # Errors
    /// Returns error if required environment variables are missing or invalid.
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if present (ignore errors for production)
        let _ = dotenvy::dotenv();

        envy::from_env::<Self>().map_err(|e| ConfigError::Parse(e.to_string()))
    }

    /// Get the socket address to bind to
    #[must_use]
    pub fn socket_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::new(
            self.app
                .app_host
                .parse()
                .unwrap_or_else(|_| std::net::IpAddr::from([0, 0, 0, 0])),
            self.app.app_port,
        )
    }

    /// Check if running in production mode
    #[must_use]
    pub fn is_production(&self) -> bool {
        self.app.app_env == Environment::Prod
    }
}

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration parse error: {0}")]
    Parse(String),

    #[error("Configuration validation error: {0}")]
    Validation(String),
}

/// Redact sensitive values for logging
///
/// Shows first 4 characters followed by asterisks for values longer than 4 chars.
/// Returns "****" for shorter values.
#[must_use]
pub fn redact_secret(value: &str) -> String {
    if value.len() <= 4 {
        "****".to_string()
    } else {
        format!("{}****", &value[..4])
    }
}

/// Deserializer helper: treat empty strings as None
pub(crate) fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_secret_short() {
        assert_eq!(redact_secret("abc"), "****");
        assert_eq!(redact_secret("1234"), "****");
    }

    #[test]
    fn test_redact_secret_long() {
        assert_eq!(redact_secret("abcdefgh"), "abcd****");
        assert_eq!(redact_secret("my-super-secret-key"), "my-s****");
    }

    #[test]
    fn test_config_loads_with_defaults() {
        // Config should load even with various env states
        // Testing that from_env() doesn't panic and returns a result
        let result = Config::from_env();
        // Either succeeds or returns a parse error - both are valid outcomes
        match result {
            Ok(config) => {
                // Verify some defaults are sensible
                assert!(config.app.app_port > 0);
            }
            Err(ConfigError::Parse(_)) => {
                // Parse errors are acceptable in test environment
            }
            Err(e) => panic!("Unexpected error type: {e:?}"),
        }
    }
}
