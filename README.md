# Barrzen Axum Kit

A reusable Axum application framework split into modular crates.

## Crates

| Crate | Description | Features |
|-------|-------------|----------|
| `barrzen-axum-core` | Config, banner, AppBuilder, middleware, handlers | - |
| `barrzen-axum-infra` | Database, cache, search, broker | `db`, `cache-moka`, `cache-redis`, `meilisearch`, `nats` |
| `barrzen-axum-obs` | Tracing and OpenTelemetry | `otel` |
| `barrzen-axum-openapi` | Swagger UI and OpenAPI docs | `openapi` |

## Usage

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

## CI/CD

Deny warnings in CI:
```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```
