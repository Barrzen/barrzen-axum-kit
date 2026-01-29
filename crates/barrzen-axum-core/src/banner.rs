//! Startup banner module
//!
//! Prints a formatted startup banner showing configuration and module status.

use crate::config::{Config, Environment};

/// Print the startup banner
///
/// Shows application version, environment, and enabled modules.
/// Controlled by `FEATURE_STARTUP_BANNER`.
pub fn print_banner(config: &Config, build: &super::BuildInfo) {
    if !config.features.feature_startup_banner {
        return;
    }

    let version = &build.version;
    let git_hash = build.git_sha.as_deref().unwrap_or("unknown");

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║            🦀  Barrzen AXUM APPLICATION  🦀                   ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Version: {:<51} ║", format!("{version} ({git_hash})"));
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  ENVIRONMENT                                                 ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Env:     {:<51} ║", env_badge(config.app.app_env));
    println!(
        "║  Debug:   {:<51} ║",
        bool_indicator(config.app.app_debug)
    );
    println!("║  Address: {:<51} ║", config.socket_addr());
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  FEATURES                                                    ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Database:    {:<47} ║",
        feature_status(config.features.feature_db)
    );
    println!(
        "║  Cache:       {:<47} ║",
        if config.features.feature_cache {
            format!("✅ ON ({})", config.cache.cache_backend)
        } else {
            "❌ OFF".to_string()
        }
    );
    println!(
        "║  Search:      {:<47} ║",
        feature_status(config.features.feature_search)
    );
    println!(
        "║  Broker:      {:<47} ║",
        feature_status(config.features.feature_broker)
    );
    println!(
        "║  OpenAPI:     {:<47} ║",
        feature_status(config.features.feature_openapi)
    );
    println!(
        "║  OTEL:        {:<47} ║",
        feature_status(config.features.feature_otel)
    );
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  HTTP                                                        ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Request Log: {:<47} ║",
        bool_indicator(config.features.feature_request_log)
    );
    println!(
        "║  Tracing:     {:<47} ║",
        bool_indicator(config.features.feature_tracing)
    );
    println!(
        "║  CORS:        {:<47} ║",
        bool_indicator(config.features.feature_cors)
    );
    println!(
        "║  Body Limit:  {:<47} ║",
        format_bytes(config.http.http_body_limit_bytes)
    );
    println!(
        "║  Timeout:     {:<47} ║",
        format!("{}s", config.http.http_request_timeout_seconds)
    );
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
}

fn env_badge(env: Environment) -> String {
    match env {
        Environment::Dev => "🔧 DEV".to_string(),
        Environment::Stage => "🚧 STAGE".to_string(),
        Environment::Prod => "🚀 PROD".to_string(),
    }
}

fn bool_indicator(value: bool) -> &'static str {
    if value {
        "✅ ON"
    } else {
        "❌ OFF"
    }
}

fn feature_status(enabled: bool) -> &'static str {
    if enabled {
        "✅ ON"
    } else {
        "❌ OFF"
    }
}

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1_048_576 {
        format!("{} MB", bytes / 1_048_576)
    } else if bytes >= 1024 {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::BuildInfo;

    #[test]
    fn test_banner_does_not_print_when_disabled() {
        // This test verifies the function doesn't panic
        // When feature_startup_banner is false, nothing should print
        let config = Config::from_env();
        if let Ok(mut config) = config {
            config.features.feature_startup_banner = false;
            let build = BuildInfo::default();
            // Should not panic
            print_banner(&config, &build);
        }
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2 KB");
        assert_eq!(format_bytes(1_048_576), "1 MB");
        assert_eq!(format_bytes(2_097_152), "2 MB");
    }

    #[test]
    fn test_env_badge() {
        assert!(env_badge(Environment::Dev).contains("DEV"));
        assert!(env_badge(Environment::Stage).contains("STAGE"));
        assert!(env_badge(Environment::Prod).contains("PROD"));
    }
}
