//! Core application settings

use serde::Deserialize;

/// Core application settings
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_app_name")]
    pub app_name: String,

    #[serde(default)]
    pub app_env: Environment,

    #[serde(default = "default_host")]
    pub app_host: String,

    #[serde(default = "default_port")]
    #[serde(deserialize_with = "crate::config::de_u16")]
    pub app_port: u16,

    #[serde(default)]
    #[serde(deserialize_with = "crate::config::de_bool")]
    pub app_debug: bool,

    #[serde(default = "default_shutdown_grace")]
    #[serde(deserialize_with = "crate::config::de_u64")]
    pub app_shutdown_grace_seconds: u64,
}

/// Environment type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    #[default]
    Dev,
    Stage,
    Prod,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dev => write!(f, "dev"),
            Self::Stage => write!(f, "stage"),
            Self::Prod => write!(f, "prod"),
        }
    }
}

fn default_app_name() -> String {
    "barrzen-app".to_string()
}
fn default_host() -> String {
    "0.0.0.0".to_string()
}
fn default_port() -> u16 {
    8080
}
fn default_shutdown_grace() -> u64 {
    10
}
