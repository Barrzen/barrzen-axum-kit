//! Standard API response types
//!
//! Provides consistent JSON envelope responses for API endpoints.

use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Standard API response wrapper
///
/// All successful responses use this format for consistency.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ApiResponse<T: Serialize> {
    /// Status string: "success" or "error"
    pub status: &'static str,
    /// HTTP status code
    pub code: u16,
    /// Human-readable message
    pub message: String,
    /// ISO 8601 timestamp
    pub timestamp: DateTime<Utc>,
    /// Request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Response data payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a success response (200 OK)
    #[must_use]
    pub fn ok(data: T, message: impl Into<String>) -> Self {
        Self {
            status: "success",
            code: StatusCode::OK.as_u16(),
            timestamp: Utc::now(),
            request_id: None,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Create a created response (201 Created)
    #[must_use]
    pub fn created(data: T, message: impl Into<String>) -> Self {
        Self {
            status: "success",
            code: StatusCode::CREATED.as_u16(),
            timestamp: Utc::now(),
            request_id: None,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Set the request ID
    #[must_use]
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Create a response with a specific status code
    #[must_use]
    pub fn with_status(status: StatusCode, data: T, message: impl Into<String>) -> Self {
        Self {
            status: "success",
            code: status.as_u16(),
            timestamp: Utc::now(),
            request_id: None,
            message: message.into(),
            data: Some(data),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::OK);
        (status, Json(self)).into_response()
    }
}

/// Error response (no data payload)
/// Standard API error structure
/// Error response (no data payload)
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ApiError {
    /// Status string: always "error"
    pub status: &'static str,
    /// HTTP status code
    pub code: u16,
    /// Human-readable error message
    pub message: String,
    /// ISO 8601 timestamp
    pub timestamp: DateTime<Utc>,
    /// Request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Optional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiError {
    fn new(code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status: "error",
            code: code.as_u16(),
            timestamp: Utc::now(),
            request_id: None,
            message: message.into(),
            details: None,
        }
    }

    /// Create a bad request error (400)
    #[must_use]
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    /// Create an unauthorized error (401)
    #[must_use]
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    /// Create a forbidden error (403)
    #[must_use]
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, message)
    }

    /// Create a not found error (404)
    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    /// Create an internal server error (500)
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    /// Create a service unavailable error (503)
    #[must_use]
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, message)
    }

    /// Set the request ID
    #[must_use]
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Set error details
    #[must_use]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

/// Result type for API handlers
pub type ApiResult<T> = Result<ApiResponse<T>, ApiError>;

/// Header name for request ID
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Helper to extract request ID from headers
#[must_use]
pub fn extract_request_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_ok() {
        let response = ApiResponse::ok("data", "Success");
        assert_eq!(response.status, "success");
        assert_eq!(response.code, 200);
        assert_eq!(response.message, "Success");
    }

    #[test]
    fn test_api_error_not_found() {
        let error = ApiError::not_found("Resource not found");
        assert_eq!(error.status, "error");
        assert_eq!(error.code, 404);
    }

    #[test]
    fn test_api_response_serializes() {
        let response = ApiResponse::ok("test", "OK");
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"success\""));
        assert!(json.contains("\"code\":200"));
        assert!(json.contains("\"timestamp\":"));
    }
}
