# barrzen-axum-obs

Observability for Barrzen Axum: tracing setup and OpenTelemetry integration.

## Features

- `otel`: Enables OpenTelemetry exporter and tracing integration
- `fast-log`: Enables the fast_log backend (log-based logging)

## Usage

```rust
use barrzen_axum_core::AppConfig;
use barrzen_axum_obs::init_tracing;

fn main() -> anyhow::Result<()> {
    let cfg = AppConfig::from_env()?;
    init_tracing(&cfg)
}
```

Default backend is `tracing`.

Set `LOG_BACKEND=fast_log` to use the fast_log backend. When enabled, `LOG_FORMAT` is ignored
and `FEATURE_OTEL=true` is not supported.

## Links

- Workspace overview: see the repository root README.
