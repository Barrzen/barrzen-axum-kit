//! Barrzen Axum OpenAPI
//!
//! OpenAPI documentation for Axum applications.

use axum::Router;

#[cfg(feature = "openapi")]
use utoipa::openapi::OpenApi;
#[cfg(feature = "openapi")]
use utoipa_swagger_ui::SwaggerUi;

/// Mount OpenAPI routes onto a router
///
/// Adds:
/// - GET /docs - Swagger UI
/// - GET /openapi.json - OpenAPI specification
///
/// Use this in your application to expose documentation.
#[cfg(feature = "openapi")]
pub fn mount(router: Router<()>, doc: OpenApi) -> Router<()> {
    router.merge(SwaggerUi::new("/docs").url("/openapi.json", doc))
}

/// No-op when openapi feature is disabled behavior depends on caller handling the feature flag
/// usually caller won't have `OpenApi` struct if feature is disabled.
///
/// So this function is strictly for when feature is enabled.
///
/// If feature is disabled, the `doc` argument type `OpenApi` won't exist.
/// So we can't really have a no-op version with the same signature easily
/// without generic wrapper or conditional compilation at call site.
///
/// Recommending call site to use `#[cfg(feature = "openapi")]`.
#[cfg(not(feature = "openapi"))]
#[cfg(not(feature = "openapi"))]
pub fn mount(_router: Router<()>, _doc: ()) -> Router<()> {
    // This signature is just a placeholder and unlikely to be used directly
    // because `doc` param would be difficult to provide.
    _router
}
