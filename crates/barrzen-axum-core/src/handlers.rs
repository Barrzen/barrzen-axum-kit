//! Core HTTP handlers
//!
//! Provides /healthz, /readyz, and /version endpoints.

use axum::{extract::State, http::HeaderMap, response::IntoResponse};
use serde::Serialize;
use std::sync::Arc;

use crate::{
    response::{extract_request_id, ApiResponse},
    BuildInfo,
};

/// Health check response data
#[derive(Debug, Serialize)]
pub struct HealthData {
    pub status: String,
}

/// Readiness check response data
#[derive(Debug, Serialize)]
pub struct ReadyData {
    pub status: String,
    pub checks: Vec<HealthCheck>,
}

/// Individual health check result
#[derive(Debug, Serialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl HealthCheck {
    /// Create an OK health check
    #[must_use]
    pub fn ok(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: "ok".to_string(),
            message: None,
        }
    }

    /// Create a failed health check
    #[must_use]
    pub fn fail(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: "fail".to_string(),
            message: Some(message.into()),
        }
    }

    /// Create a skipped health check
    #[must_use]
    pub fn skip(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: "skip".to_string(),
            message: Some(reason.into()),
        }
    }
}

/// Version info response data
#[derive(Debug, Serialize)]
pub struct VersionData {
    pub name: String,
    pub version: String,
    pub git_hash: Option<String>,
    pub rust_version: String,
}

/// Application state for core handlers
#[derive(Clone)]
pub struct CoreState {
    pub build_info: Arc<BuildInfo>,
    pub ready_checker: Option<Arc<dyn ReadyChecker>>,
    pub feature_response_envelope: bool,
}

impl CoreState {
    /// Create new core state
    #[must_use]
    pub fn new(build_info: BuildInfo, feature_response_envelope: bool) -> Self {
        Self {
            build_info: Arc::new(build_info),
            ready_checker: None,
            feature_response_envelope,
        }
    }

    /// Add a ready checker
    #[must_use]
    pub fn with_ready_checker(mut self, checker: Arc<dyn ReadyChecker>) -> Self {
        self.ready_checker = Some(checker);
        self
    }
}

/// Trait for readiness checking
///
/// Implement this for your Infra struct to provide health checks.
#[async_trait::async_trait]
pub trait ReadyChecker: Send + Sync {
    /// Get readiness report
    async fn ready_checks(&self) -> Vec<HealthCheck>;
}

/// GET /healthz - Basic liveness check (always 200 OK)
pub async fn healthz(headers: HeaderMap, State(state): State<CoreState>) -> impl IntoResponse {
    let request_id = extract_request_id(&headers);
    let data = HealthData {
        status: "ok".to_string(),
    };

    if state.feature_response_envelope {
        let mut response = ApiResponse::ok(data, "Service is healthy");
        if let Some(rid) = request_id {
            response = response.with_request_id(rid);
        }
        response.into_response()
    } else {
        axum::Json(data).into_response()
    }
}

/// GET /readyz - Readiness check (checks enabled dependencies)
pub async fn readyz(headers: HeaderMap, State(state): State<CoreState>) -> impl IntoResponse {
    let request_id = extract_request_id(&headers);

    let checks = if let Some(ref checker) = state.ready_checker {
        checker.ready_checks().await
    } else {
        vec![HealthCheck::skip("infra", "not configured")]
    };

    let all_ok = checks.iter().all(|c| c.status == "ok" || c.status == "skip");

    let data = ReadyData {
        status: if all_ok {
            "ok".to_string()
        } else {
            "degraded".to_string()
        },
        checks,
    };

    if state.feature_response_envelope {
        let message = if all_ok {
            "Service is ready"
        } else {
            "Service is degraded"
        };

        let mut response = ApiResponse::ok(data, message);
        if let Some(rid) = request_id {
            response = response.with_request_id(rid);
        }
        response.into_response()
    } else {
        // For readiness, we might want to set status code even without envelope?
        // Standard behavior: 200 OK or 503 if strict?
        // Current implementation logic always returned 200 OK with status inside body.
        // We stick to that for raw JSON too unless specific requirement.
        axum::Json(data).into_response()
    }
}

/// GET /version - Build and version info
pub async fn version(headers: HeaderMap, State(state): State<CoreState>) -> impl IntoResponse {
    let request_id = extract_request_id(&headers);
    let build = &state.build_info;

    let data = VersionData {
        name: build.name.clone(),
        version: build.version.clone(),
        git_hash: build.git_sha.clone(),
        rust_version: build.rust_version.clone(),
    };

    if state.feature_response_envelope {
        let mut response = ApiResponse::ok(data, "Version information");
        if let Some(rid) = request_id {
            response = response.with_request_id(rid);
        }
        response.into_response()
    } else {
        axum::Json(data).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_ok() {
        let check = HealthCheck::ok("test");
        assert_eq!(check.status, "ok");
        assert!(check.message.is_none());
    }

    #[test]
    fn test_health_check_fail() {
        let check = HealthCheck::fail("test", "connection refused");
        assert_eq!(check.status, "fail");
        assert!(check.message.is_some());
    }

    #[test]
    fn test_core_state_creation() {
        let build = BuildInfo::new("test", "1.0.0", None, "1.75.0", None);
        let state = CoreState::new(build, true);
        assert!(state.ready_checker.is_none());
    }
}
