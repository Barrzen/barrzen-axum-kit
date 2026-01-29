//! Barrzen Axum Core
//!
//! Core components for building Axum applications:
//! - Configuration and environment parsing
//! - Startup banner
//! - Build information
//! - AppBuilder for router and middleware composition
//! - Standard API response types
//! - Core endpoints: /healthz, /readyz, /version

pub mod app_builder;
pub mod banner;
pub mod build_info;
pub mod config;
pub mod handlers;
pub mod response;

pub use app_builder::AppBuilder;
pub use build_info::BuildInfo;
pub use config::{
    AppConfig, BannerConfig, CacheBackend, CacheConfig, Config, ConfigError, CorsConfig,
    Environment, FeatureFlags, HttpConfig, LogFormat, LoggingConfig,
};
pub use handlers::{CoreState, HealthCheck, ReadyChecker};
pub use response::{ApiError, ApiResponse, ApiResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_crate_compiles() {
        // Basic smoke test - verify re-exports work
        let _ = Environment::Dev;
        let _ = LogFormat::Pretty;
        let _ = CacheBackend::Moka;
        let _ = BuildInfo::default();
    }
}
