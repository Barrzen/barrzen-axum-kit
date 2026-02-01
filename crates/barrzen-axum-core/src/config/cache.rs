//! Cache configuration

use serde::Deserialize;

use super::empty_string_as_none;

/// Cache configuration
#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    #[serde(default)]
    pub cache_backend: CacheBackend,

    #[serde(default = "default_cache_ttl")]
    #[serde(deserialize_with = "crate::config::de_u64")]
    pub cache_ttl_seconds: u64,

    #[serde(default = "default_cache_max_entries")]
    #[serde(deserialize_with = "crate::config::de_u64")]
    pub cache_max_entries: u64,

    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub cache_redis_url: Option<String>,

    #[serde(default = "default_redis_pool_size")]
    #[serde(deserialize_with = "crate::config::de_usize")]
    pub cache_redis_pool_size: usize,

    #[serde(default = "default_connect_timeout")]
    #[serde(deserialize_with = "crate::config::de_u64")]
    pub cache_redis_connect_timeout_seconds: u64,
}

/// Cache backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CacheBackend {
    None,
    #[default]
    Moka,
    Redis,
}

impl std::fmt::Display for CacheBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Moka => write!(f, "moka"),
            Self::Redis => write!(f, "redis"),
        }
    }
}

fn default_cache_ttl() -> u64 {
    300
}
fn default_cache_max_entries() -> u64 {
    50_000
}
fn default_redis_pool_size() -> usize {
    20
}
fn default_connect_timeout() -> u64 {
    5
}
