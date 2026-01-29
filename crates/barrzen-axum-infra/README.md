# barrzen-axum-infra

Infrastructure integrations for Barrzen Axum: database, cache, search, and broker.

## Features

- `db`: SeaORM database connection
- `cache-moka`: Moka in-memory cache
- `cache-redis`: Redis/Valkey cache via deadpool
- `meilisearch`: Meilisearch client
- `nats`: NATS broker client

## Usage

```rust
use barrzen_axum_core::AppConfig;
use barrzen_axum_infra::Infra;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = AppConfig::from_env()?;
    let infra = Infra::init(&cfg).await?;
    // use infra in AppBuilder
    Ok(())
}
```

## Links

- Workspace overview: see the repository root README.
