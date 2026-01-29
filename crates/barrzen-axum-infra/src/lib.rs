//! Barrzen Axum Infra
//!
//! Infrastructure integrations for Axum applications.
//!
//! Manages connections to:
//! - Database (SeaORM)
//! - Cache (Moka/Redis)
//! - Search (Meilisearch)
//! - Broker (NATS)

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use barrzen_axum_core::{Config, HealthCheck, ReadyChecker};

/// Infrastructure container
#[derive(Clone, Default)]
pub struct Infra {
    // Database
    #[cfg(feature = "db")]
    pub db: Option<sea_orm::DatabaseConnection>,

    // Cache
    #[cfg(any(feature = "cache-moka", feature = "cache-redis"))]
    pub cache: Option<Arc<dyn Cache + Send + Sync>>,

    // Search
    #[cfg(feature = "meilisearch")]
    pub search: Option<meilisearch_sdk::client::Client>,

    // Broker
    #[cfg(feature = "nats")]
    pub broker: Option<async_nats::Client>,
}

impl Infra {
    /// Initialize infrastructure based on configuration
    ///
    /// # Errors
    /// Returns error if a feature is enabled at runtime but not compiled,
    /// or if connection setup fails.
    pub async fn init(config: &Config) -> anyhow::Result<Self> {
        let infra = Self::default();

        // Database
        if config.features.feature_db {
            #[cfg(feature = "db")]
            {
                infra.db = Some(init_db(config).await?);
            }
            #[cfg(not(feature = "db"))]
            {
                anyhow::bail!("FEATURE_DB is enabled but 'db' cargo feature is disabled");
            }
        }

        // Cache
        if config.features.feature_cache {
            // Moka
            if matches!(config.cache.cache_backend, barrzen_axum_core::CacheBackend::Moka) {
                #[cfg(feature = "cache-moka")]
                {
                    infra.cache = Some(init_moka_cache(config));
                }
                #[cfg(not(feature = "cache-moka"))]
                {
                    anyhow::bail!("Cache backend 'moka' selected but 'cache-moka' cargo feature is disabled");
                }
            }
            // Redis
            if matches!(config.cache.cache_backend, barrzen_axum_core::CacheBackend::Redis) {
                #[cfg(feature = "cache-redis")]
                {
                    infra.cache = Some(init_redis_cache(config).await?);
                }
                #[cfg(not(feature = "cache-redis"))]
                {
                    anyhow::bail!("Cache backend 'redis' selected but 'cache-redis' cargo feature is disabled");
                }
            }
        }

        // Search
        if config.features.feature_search {
            #[cfg(feature = "meilisearch")]
            {
                // Init meilisearch (placeholder for now)
                // infra.search = Some(init_meilisearch(config).await?);
            }
            #[cfg(not(feature = "meilisearch"))]
            {
                anyhow::bail!("FEATURE_SEARCH is enabled but 'meilisearch' cargo feature is disabled");
            }
        }

        // Broker
        if config.features.feature_broker {
            #[cfg(feature = "nats")]
            {
                // Init nats (placeholder)
                // infra.broker = Some(init_nats(config).await?);
            }
            #[cfg(not(feature = "nats"))]
            {
                anyhow::bail!("FEATURE_BROKER is enabled but 'nats' cargo feature is disabled");
            }
        }

        Ok(infra)
    }
}

#[async_trait::async_trait]
impl ReadyChecker for Infra {
    async fn ready_checks(&self) -> Vec<HealthCheck> {
        let mut checks = Vec::new();

        // Database Check
        #[cfg(feature = "db")]
        if let Some(db) = &self.db {
            match db.ping().await {
                Ok(_) => checks.push(HealthCheck::ok("database")),
                Err(e) => checks.push(HealthCheck::fail("database", e.to_string())),
            }
        } else {
            checks.push(HealthCheck::skip("database", "disabled"));
        }
        #[cfg(not(feature = "db"))]
        checks.push(HealthCheck::skip("database", "not-compiled"));

        // Cache Check
        #[cfg(any(feature = "cache-moka", feature = "cache-redis"))]
        if let Some(cache) = &self.cache {
             match cache.ping().await {
                 Ok(_) => checks.push(HealthCheck::ok("cache")),
                 Err(e) => checks.push(HealthCheck::fail("cache", e.to_string())),
             }
        } else {
             checks.push(HealthCheck::skip("cache", "disabled"));
        }
        #[cfg(not(any(feature = "cache-moka", feature = "cache-redis")))]
        checks.push(HealthCheck::skip("cache", "not-compiled"));

        checks
    }
}

// Internal initializers

#[cfg(feature = "db")]
async fn init_db(config: &Config) -> anyhow::Result<sea_orm::DatabaseConnection> {
    use sea_orm::{ConnectOptions, Database};
    
    // We would need DATABASE_URL logic here. 
    // Assuming config might have it or we load it from env directly since it's sensitive.
    // Core config didn't have specific DB config struct yet.
    // For now, let's assume DATABASE_URL env var.
    let url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    
    let mut opt = ConnectOptions::new(url);
    opt.max_connections(100)
       .min_connections(5)
       .connect_timeout(Duration::from_secs(10))
       .acquire_timeout(Duration::from_secs(10))
       .idle_timeout(Duration::from_secs(10))
       .max_lifetime(Duration::from_secs(1800))
       .sqlx_logging(false);

    let db = Database::connect(opt).await?;
    Ok(db)
}

#[cfg(feature = "cache-moka")]
fn init_moka_cache(config: &Config) -> Arc<dyn Cache + Send + Sync> {
    // Placeholder Moka init
    Arc::new(MokaCacheStub)
}

#[cfg(feature = "cache-redis")]
async fn init_redis_cache(config: &Config) -> anyhow::Result<Arc<dyn Cache + Send + Sync>> {
    // Placeholder Redis init
    Ok(Arc::new(RedisCacheStub))
}

// Cache Abstraction (Stub for now)
#[async_trait::async_trait]
pub trait Cache {
    async fn ping(&self) -> anyhow::Result<()>;
}

struct MokaCacheStub;
#[async_trait::async_trait]
impl Cache for MokaCacheStub {
    async fn ping(&self) -> anyhow::Result<()> { Ok(()) }
}

struct RedisCacheStub;
#[async_trait::async_trait]
impl Cache for RedisCacheStub {
    async fn ping(&self) -> anyhow::Result<()> { Ok(()) }
}
