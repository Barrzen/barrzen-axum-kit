# Barrzen Axum Kit - Agent Notes

## Purpose
- Workspace of reusable Axum crates: core, infra, obs, openapi.
- Intended to be consumed by the Barrzen Axum Template and generated services.

## Crate map
- `crates/barrzen-axum-core`: config/env parsing, banner, build info, `AppBuilder`, API response types, core handlers.
- `crates/barrzen-axum-infra`: infra container + `ReadyChecker` impl; DB/cache/search/broker behind cargo features.
- `crates/barrzen-axum-obs`: tracing/log setup with optional OpenTelemetry and fast_log backend.
- `crates/barrzen-axum-openapi`: Swagger UI + OpenAPI mounting helpers.

## Runtime flow (typical)
1) `Config::from_env()` loads `.env` (via `dotenvy`) and env vars (via `envy`).
2) `barrzen_axum_obs::init_tracing(&config)` sets logging/tracing.
3) `Infra::init(&config)` initializes enabled services.
4) `AppBuilder::new(config, build_info)` sets core routes and middleware, then `serve()`.

## Core routes and middleware
- Core routes: `GET /healthz`, `GET /readyz`, `GET /version` in `crates/barrzen-axum-core/src/handlers.rs`.
- Middleware stack (from `AppBuilder`): compression, security headers, body limit, optional tracing + request log, sensitive headers, request ID propagate/set.
- `feature_response_envelope` only wraps core handlers; user handlers must opt into `ApiResponse` manually.

## Config and flags
- Runtime toggles live in `FeatureFlags` (env `FEATURE_*`).
- Compile-time cargo features control what code is built; runtime flags decide what is initialized.
- Logging supports `LOG_BACKEND=tracing|fast_log`; fast_log is incompatible with `FEATURE_OTEL=true`.

## Known gaps / improvement targets
- `HttpConfig::http_request_timeout_seconds` is not enforced by a timeout layer.
- `CorsConfig` exists but no CORS middleware is applied.
- `request_log_headers_allowlist` is unused; request logging currently logs only method/path/status/latency.
- Infra DB init uses `DATABASE_URL`, while template env files use `DB_URL` and other DB_* fields.
- Search and broker initialization are placeholders.
- `/readyz` always returns HTTP 200 even when degraded.

## Useful commands
- `cargo test --workspace`
- `cargo test --workspace --all-features`
- `scripts/test_matrix.sh`
