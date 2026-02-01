# Barrzen Axum Kit

A reusable Axum application framework split into modular crates.

[![tests](https://github.com/Barrzen/barrzen-axum-kit/actions/workflows/ci.yml/badge.svg)](https://github.com/Barrzen/barrzen-axum-kit/actions/workflows/ci.yml)

GitHub repo: `https://github.com/Barrzen/barrzen-axum-kit`

## Crates

| Crate | Description | Features | Crates.io | Docs |
|-------|-------------|----------|-----------|------|
| `barrzen-axum-core` | Config, banner, AppBuilder, middleware, handlers | - | https://crates.io/crates/barrzen-axum-core | https://docs.rs/barrzen-axum-core |
| `barrzen-axum-infra` | Database, cache, search, broker | `db`, `cache-moka`, `cache-redis`, `meilisearch`, `nats` | https://crates.io/crates/barrzen-axum-infra | https://docs.rs/barrzen-axum-infra |
| `barrzen-axum-obs` | Tracing and OpenTelemetry | `otel` | https://crates.io/crates/barrzen-axum-obs | https://docs.rs/barrzen-axum-obs |
| `barrzen-axum-openapi` | Swagger UI and OpenAPI docs | `openapi` | https://crates.io/crates/barrzen-axum-openapi | https://docs.rs/barrzen-axum-openapi |

## Installation

Pick the crates you need:

```toml
[dependencies]
barrzen-axum-core = "0.1.4"

# Optional crates
barrzen-axum-infra = { version = "0.1.4", features = ["db", "cache-redis"] }
barrzen-axum-obs = { version = "0.1.4", features = ["otel"] }
barrzen-axum-openapi = { version = "0.1.4", features = ["openapi"] }
```

Or using cargo:

```bash
cargo add barrzen-axum-core
cargo add barrzen-axum-infra --features db,cache-redis
cargo add barrzen-axum-obs --features otel
cargo add barrzen-axum-openapi --features openapi
```

## Quick start

```rust
use barrzen_axum_core::{AppBuilder, AppConfig, BuildInfo};
use barrzen_axum_infra::Infra;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = AppConfig::from_env()?;
    let build = BuildInfo::from_env_or_defaults();
    let infra = Infra::init(&cfg).await?;
    
    AppBuilder::new(cfg, build)
        .with_ready_checker(infra)
        .merge(my_app::router())
        .serve()
        .await
}
```

## Docs

- Rust docs: https://docs.rs/barrzen-axum-core
- OpenAPI docs: https://docs.rs/barrzen-axum-openapi
- Observability: https://docs.rs/barrzen-axum-obs
- Infra: https://docs.rs/barrzen-axum-infra

## Development

```bash
# Check all crates
cargo check

# Run tests
cargo test

# Lint
cargo clippy --all-targets -- -D warnings
```

## Compile-time vs Runtime Features

- **Cargo features** control what code is compiled into the binary
- **FEATURE_* env vars** control what is initialized at runtime

Example: You can compile with `db` feature but set `FEATURE_DB=false` to skip database initialization.

## Logging

- Default `LOG_FORMAT` is `compact` (single‑line, no color).
- Set `LOG_FORMAT=pretty` or `LOG_FORMAT=json` if you prefer those formats.

## Banner

- Set `BANNER_SHOW_ENV_VARS=true` to print all environment variables in the startup banner.
- Set `BANNER_SHOW_SECRETS=true` to print full values (otherwise values are redacted).

## CI/CD

Deny warnings in CI:
```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```
