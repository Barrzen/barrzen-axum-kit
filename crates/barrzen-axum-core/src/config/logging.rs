//! Logging configuration

use serde::Deserialize;

use super::empty_string_as_none;

/// Logging configuration
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default)]
    pub log_format: LogFormat,

    #[serde(default)]
    pub log_include_target: bool,

    #[serde(default)]
    pub log_include_fileline: bool,

    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub request_log_headers_allowlist: Option<String>,

    #[serde(default = "default_headers_denylist")]
    pub request_log_headers_denylist: String,
}

/// Log format type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Pretty,
    Json,
}

fn default_log_level() -> String {
    "info".to_string()
}
fn default_headers_denylist() -> String {
    "authorization,cookie,set-cookie,x-api-key".to_string()
}
