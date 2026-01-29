# barrzen-axum-core

Core components for Barrzen Axum applications: configuration, banner, AppBuilder, middleware, and handlers.

## Features

- `openapi`: Enables OpenAPI-related helpers that integrate with the openapi crate.

## Usage

```rust
use barrzen_axum_core::{AppBuilder, AppConfig, BuildInfo};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = AppConfig::from_env()?;
    let build = BuildInfo::from_env_or_defaults();

    AppBuilder::new(cfg, build)
        .merge(my_app::router())
        .serve()
        .await
}
```

## Links

- Workspace overview: see the repository root README.
