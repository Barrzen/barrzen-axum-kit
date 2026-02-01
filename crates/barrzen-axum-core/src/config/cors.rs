//! CORS configuration

use serde::Deserialize;

use super::empty_string_as_none;

/// CORS configuration
#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub cors_allow_origins: Option<String>,

    #[serde(default = "default_cors_methods")]
    pub cors_allow_methods: String,

    #[serde(default = "default_cors_headers")]
    pub cors_allow_headers: String,

    #[serde(default)]
    pub cors_allow_credentials: bool,

    #[serde(default = "default_cors_max_age")]
    #[serde(deserialize_with = "crate::config::de_u64")]
    pub cors_max_age_seconds: u64,
}

impl CorsConfig {
    /// Parse allowed origins into a vector
    #[must_use]
    pub fn origins(&self) -> Vec<String> {
        self.cors_allow_origins
            .as_ref()
            .map(|s| {
                s.split(',')
                    .map(|o| o.trim().to_string())
                    .filter(|o| !o.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Parse allowed methods
    #[must_use]
    pub fn methods(&self) -> Vec<String> {
        self.cors_allow_methods
            .split(',')
            .map(|m| m.trim().to_string())
            .filter(|m| !m.is_empty())
            .collect()
    }

    /// Parse allowed headers
    #[must_use]
    pub fn headers(&self) -> Vec<String> {
        self.cors_allow_headers
            .split(',')
            .map(|h| h.trim().to_string())
            .filter(|h| !h.is_empty())
            .collect()
    }
}

fn default_cors_methods() -> String {
    "GET,POST,PUT,PATCH,DELETE,OPTIONS".to_string()
}
fn default_cors_headers() -> String {
    "content-type,authorization".to_string()
}
fn default_cors_max_age() -> u64 {
    600
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_origins_parsing() {
        let cors = CorsConfig {
            cors_allow_origins: Some("http://localhost:3000, http://example.com".to_string()),
            cors_allow_methods: "GET,POST".to_string(),
            cors_allow_headers: "content-type".to_string(),
            cors_allow_credentials: false,
            cors_max_age_seconds: 600,
        };

        assert_eq!(
            cors.origins(),
            vec!["http://localhost:3000", "http://example.com"]
        );
        assert_eq!(cors.methods(), vec!["GET", "POST"]);
    }
}
