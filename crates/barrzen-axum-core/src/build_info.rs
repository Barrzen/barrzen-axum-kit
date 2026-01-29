//! Build information
//!
//! Contains version, git info, and build timestamp.

use serde::Serialize;

/// Build information for the application
#[derive(Debug, Clone, Default, Serialize)]
pub struct BuildInfo {
    /// Crate/package name
    pub name: String,
    /// Crate version from Cargo.toml
    pub version: String,
    /// Git commit SHA (short form)
    pub git_sha: Option<String>,
    /// Rust compiler version
    pub rust_version: String,
    /// Build timestamp (ISO 8601)
    pub build_time: Option<String>,
}

impl BuildInfo {
    /// Create new build info
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        git_sha: Option<String>,
        rust_version: impl Into<String>,
        build_time: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            git_sha,
            rust_version: rust_version.into(),
            build_time,
        }
    }

    /// Create build info from environment variables or use defaults
    ///
    /// Commonly used environment variables:
    /// - `CARGO_PKG_NAME` - package name
    /// - `CARGO_PKG_VERSION` - package version
    /// - `GIT_SHA` - git commit hash
    /// - `BUILD_TIME` - build timestamp
    #[must_use]
    pub fn from_env_or_defaults() -> Self {
        Self {
            name: std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "unknown".to_string()),
            version: std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string()),
            git_sha: std::env::var("GIT_SHA").ok(),
            rust_version: env!("CARGO_PKG_RUST_VERSION").to_string(),
            build_time: std::env::var("BUILD_TIME").ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_info_new() {
        let info = BuildInfo::new(
            "test-app",
            "1.0.0",
            Some("abc123".to_string()),
            "1.75.0",
            Some("2024-01-01T00:00:00Z".to_string()),
        );
        assert_eq!(info.name, "test-app");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.git_sha, Some("abc123".to_string()));
    }

    #[test]
    fn test_build_info_default() {
        let info = BuildInfo::default();
        assert!(info.name.is_empty());
        assert!(info.version.is_empty());
    }

    #[test]
    fn test_build_info_from_env() {
        let info = BuildInfo::from_env_or_defaults();
        // Should not panic and should have some values
        assert!(!info.version.is_empty() || info.version == "0.0.0");
    }

    #[test]
    fn test_build_info_serializes() {
        let info = BuildInfo::new("app", "1.0.0", None, "1.75.0", None);
        let json = serde_json::to_string(&info);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("\"name\":\"app\""));
        assert!(json_str.contains("\"version\":\"1.0.0\""));
    }
}
