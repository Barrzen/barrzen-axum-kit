//! Feature toggles (runtime)

use serde::Deserialize;

fn default_true() -> bool {
    true
}

/// Feature toggles (runtime)
///
/// These control what modules are initialized at runtime.
/// Separate from Cargo features which control compile-time inclusion.
#[derive(Debug, Clone, Deserialize)]
pub struct FeatureFlags {
    #[serde(default = "default_true")]
    pub feature_startup_banner: bool,

    #[serde(default)]
    pub feature_db: bool,

    #[serde(default = "default_true")]
    pub feature_cache: bool,

    #[serde(default)]
    pub feature_search: bool,

    #[serde(default)]
    pub feature_broker: bool,

    #[serde(default)]
    pub feature_openapi: bool,

    #[serde(default = "default_true")]
    pub feature_request_log: bool,

    #[serde(default = "default_true")]
    pub feature_tracing: bool,

    #[serde(default)]
    pub feature_otel: bool,

    #[serde(default)]
    pub feature_cors: bool,

    #[serde(default)]
    pub feature_session: bool,

    #[serde(default = "default_true")]
    pub feature_response_envelope: bool,
}
