//! AppBuilder for router and middleware composition
//!
//! Provides a builder pattern for constructing Axum applications.

use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use axum::{
    http::{HeaderName, HeaderValue, Method},
    http::Request,
    Router,
};
use tokio::net::TcpListener;
use tower::{Layer, Service};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    sensitive_headers::SetSensitiveRequestHeadersLayer,
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};

use crate::{
    config::{Config, LogBackend},
    handlers::{self, CoreState, ReadyChecker},
    BuildInfo,
};
/// Header name for request ID
pub static REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Application builder
///
/// Constructs an Axum application with standard middleware and routes.
pub struct AppBuilder {
    config: Config,
    build_info: BuildInfo,
    ready_checker: Option<Arc<dyn ReadyChecker>>,
    user_router: Option<Router<CoreState>>,
    user_stateless_router: Option<Router<()>>,
}

impl AppBuilder {
    /// Create a new app builder
    #[must_use]
    pub fn new(config: Config, build_info: BuildInfo) -> Self {
        Self {
            config,
            build_info,
            ready_checker: None,
            user_router: None,
            user_stateless_router: None,
        }
    }

    /// Add infrastructure for health checks
    #[must_use]
    pub fn with_ready_checker(mut self, checker: impl ReadyChecker + 'static) -> Self {
        self.ready_checker = Some(Arc::new(checker));
        self
    }

    /// Merge user routes (stateful)
    #[must_use]
    pub fn merge(mut self, router: Router<CoreState>) -> Self {
        self.user_router = Some(router);
        self
    }
    
    /// Merge stateless routes (e.g. Swagger UI, unmodified static handlers)
    #[must_use]
    pub fn merge_stateless(mut self, router: Router<()>) -> Self {
        self.user_stateless_router = Some(router);
        self
    }

    /// Build the router with all middleware
    #[must_use]
    pub fn build(self) -> Router {
        let Self {
            config,
            build_info,
            ready_checker,
            user_router,
            user_stateless_router,
        } = self;

        let state = CoreState::new(build_info, config.features.feature_response_envelope);
        let state = if let Some(checker) = ready_checker {
            state.with_ready_checker(checker)
        } else {
            state
        };

        // Start with core routes
        let mut app: Router<CoreState> = Router::new()
            .route("/healthz", axum::routing::get(handlers::healthz))
            .route("/readyz", axum::routing::get(handlers::readyz))
            .route("/version", axum::routing::get(handlers::version));

        // Merge stateless routes as fallback
        if let Some(router) = user_stateless_router {
            app = app.fallback_service(router);
        }

        // Merge user routes
        if let Some(router) = user_router {
            app = app.merge(router);
        }

        // Apply middleware
        app = apply_middleware(app, &config);

        app.with_state(state)
    }

    /// Serve the application
    ///
    /// # Errors
    /// Returns error if binding or serving fails.
    pub async fn serve(self) -> anyhow::Result<()> {
        let addr = self.config.socket_addr();
        let grace_seconds = self.config.app.app_shutdown_grace_seconds;

        // Print banner
        crate::banner::print_banner(&self.config, &self.build_info);

        let app = self.build();
        let listener = TcpListener::bind(addr).await?;

        tracing::info!("Server listening on http://{}", addr);

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal(grace_seconds))
            .await?;

        tracing::info!("Server shutdown complete");

        Ok(())
    }
}

fn apply_middleware(router: Router<CoreState>, config: &Config) -> Router<CoreState> {
    // Sensitive headers
    let sensitive_headers: Vec<HeaderName> = config
        .logging
        .request_log_headers_denylist
        .split(',')
        .filter_map(|h| h.trim().parse().ok())
        .collect();

    // Start building middleware stack (applied in reverse order)
    let router = router.layer(CompressionLayer::new());

    // Security headers
    let router = apply_security_headers(router);

    // Body limit
    let router = router.layer(RequestBodyLimitLayer::new(config.http.http_body_limit_bytes));

    // Tracing layer (conditional)
    let router = if config.features.feature_tracing {
        // Keep spans for tracing, but disable default response logs to avoid duplicates.
        router.layer(
            TraceLayer::new_for_http()
                .on_request(())
                .on_response(())
                .on_failure(()),
        )
    } else {
        router
    };

    // Request logging (conditional)
    let router = if config.features.feature_request_log {
        router.layer(RequestLogLayer::new(config.logging.log_backend))
    } else {
        router
    };

    // Sensitive headers protection
    let router = router.layer(SetSensitiveRequestHeadersLayer::new(sensitive_headers));

    // Request ID layers
    let router = router.layer(PropagateRequestIdLayer::new(REQUEST_ID_HEADER.clone()));
    let router = router.layer(SetRequestIdLayer::new(
        REQUEST_ID_HEADER.clone(),
        MakeRequestUuid,
    ));

    // CORS (conditional)
    let router = if config.features.feature_cors {
        router.layer(build_cors_layer(config))
    } else {
        router
    };

    router
}

fn build_cors_layer(config: &Config) -> CorsLayer {
    let mut cors = CorsLayer::new()
        .max_age(Duration::from_secs(config.cors.cors_max_age_seconds));

    if config.cors.cors_allow_credentials {
        cors = cors.allow_credentials(true);
    }

    let methods: Vec<Method> = config
        .cors
        .methods()
        .into_iter()
        .filter_map(|method| Method::from_bytes(method.as_bytes()).ok())
        .collect();
    if !methods.is_empty() {
        cors = cors.allow_methods(methods);
    }

    let headers: Vec<HeaderName> = config
        .cors
        .headers()
        .into_iter()
        .filter_map(|header| HeaderName::from_bytes(header.as_bytes()).ok())
        .collect();
    if !headers.is_empty() {
        cors = cors.allow_headers(headers);
    }

    let origins: Vec<HeaderValue> = config
        .cors
        .origins()
        .into_iter()
        .filter_map(|origin| HeaderValue::from_str(&origin).ok())
        .collect();
    if !origins.is_empty() {
        cors = cors.allow_origin(origins);
    }

    cors
}

#[derive(Clone, Copy)]
struct RequestLogLayer {
    backend: LogBackend,
}

impl RequestLogLayer {
    fn new(backend: LogBackend) -> Self {
        Self { backend }
    }
}

impl<S> Layer<S> for RequestLogLayer {
    type Service = RequestLogService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestLogService {
            inner,
            backend: self.backend,
        }
    }
}

#[derive(Clone)]
struct RequestLogService<S> {
    inner: S,
    backend: LogBackend,
}

impl<S, B> Service<Request<B>> for RequestLogService<S>
where
    S: Service<Request<B>, Response = axum::response::Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let mut inner = self.inner.clone();
        let backend = self.backend;

        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let request_id = req
            .headers()
            .get(&REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let start = Instant::now();

        Box::pin(async move {
            let response = inner.call(req).await?;
            let latency_ms = start.elapsed().as_millis() as u64;

            match backend {
                LogBackend::Tracing => {
                    tracing::info!(
                        request_id = %request_id,
                        method = %method,
                        path = %path,
                        status = response.status().as_u16(),
                        latency_ms = latency_ms,
                        "request completed"
                    );
                }
                LogBackend::FastLog => {
                    log::info!(
                        "request completed request_id={} method={} path={} status={} latency_ms={}",
                        request_id,
                        method,
                        path,
                        response.status().as_u16(),
                        latency_ms
                    );
                }
            }

            Ok(response)
        })
    }
}

/// Apply security-related response headers
fn apply_security_headers(router: Router<CoreState>) -> Router<CoreState> {
    router
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
}

/// Graceful shutdown signal handler
async fn shutdown_signal(grace_seconds: u64) {
    use tokio::signal;

    let ctrl_c = async {
        if let Err(err) = signal::ctrl_c().await {
            tracing::error!("failed to install Ctrl+C handler: {}", err);
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(err) => {
                tracing::error!("failed to install SIGTERM handler: {}", err);
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
            tracing::info!("Received Ctrl+C, starting graceful shutdown ({}s grace period)...", grace_seconds);
        },
        () = terminate => {
            tracing::info!("Received SIGTERM, starting graceful shutdown ({}s grace period)...", grace_seconds);
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_app_builder_creates_router() {
        // Create config using JSON deserialization to avoid manual construction of private fields
        let config: Config = serde_json::from_str(r#"{
            "app_name": "test-app",
            "app_env": "dev",
            "app_host": "127.0.0.1",
            "app_port": 0,
            "app_debug": true,
            "app_shutdown_grace_seconds": 1,
            "feature_startup_banner": false,
            "feature_db": false,
            "feature_cache": false,
            "feature_search": false,
            "feature_broker": false,
            "feature_openapi": false,
            "feature_request_log": false,
            "feature_tracing": false,
            "feature_otel": false,
            "feature_cors": false,
            "feature_session": false,
            "feature_response_envelope": true,
            "http_body_limit_bytes": 1024,
            "http_request_timeout_seconds": 1,
            "log_level": "info",
            "log_format": "pretty",
            "log_include_target": false,
            "log_include_fileline": false,
            "request_log_headers_denylist": "",
            "cache_backend": "none",
            "cache_ttl_seconds": 60,
            "cache_max_entries": 1000,
            "cache_redis_pool_size": 1,
            "cache_redis_connect_timeout_seconds": 1,
            "cors_allow_methods": "GET",
            "cors_allow_headers": "content-type",
            "cors_allow_credentials": false,
            "cors_max_age_seconds": 60,
            "banner_show_secrets": false,
            "banner_show_env_vars": false
        }"#).expect("Failed to create test config");

        let build = BuildInfo::new("test", "1.0.0", None, "1.75.0", None);
        let app = AppBuilder::new(config, build).build();

        // Test healthz endpoint
        let response = app
            .oneshot(Request::builder().uri("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
    }
}
