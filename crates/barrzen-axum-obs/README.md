# barrzen-axum-obs

Observability for Barrzen Axum: tracing setup and OpenTelemetry integration.

## Features

- `otel`: Enables OpenTelemetry exporter and tracing integration

## Usage

```rust
use barrzen_axum_core::AppConfig;
use barrzen_axum_obs::init_tracing;

fn main() -> anyhow::Result<()> {
    let cfg = AppConfig::from_env()?;
    init_tracing(&cfg)
}
```

## Links

- Workspace overview: see the repository root README.
