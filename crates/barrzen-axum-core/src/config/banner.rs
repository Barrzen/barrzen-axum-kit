//! Banner display configuration

use serde::Deserialize;

use super::empty_string_as_none;

/// Banner display configuration
#[derive(Debug, Clone, Deserialize)]
pub struct BannerConfig {
    #[serde(default)]
    #[serde(deserialize_with = "crate::config::de_bool")]
    pub banner_show_secrets: bool,

    #[serde(default)]
    #[serde(deserialize_with = "crate::config::de_bool")]
    pub banner_show_env_vars: bool,

    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub banner_env_allowlist: Option<String>,
}
