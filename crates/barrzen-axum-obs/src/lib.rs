//! Barrzen Axum Observability
//!
//! Handles tracing setup and OpenTelemetry integration.

use barrzen_axum_core::{Config, LogBackend, LogFormat};
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

#[cfg(feature = "otel")]
use tracing_subscriber::Layer;
#[cfg(feature = "otel")]
use std::sync::OnceLock;

#[cfg(feature = "otel")]
static OTEL_PROVIDER: OnceLock<opentelemetry_sdk::trace::SdkTracerProvider> = OnceLock::new();

/// Initialize tracing based on configuration
///
/// # Errors
/// Returns error if tracing subscriber setup fails.
pub fn init_tracing(config: &Config) -> anyhow::Result<()> {
    match config.logging.log_backend {
        LogBackend::Tracing => {
            let env_filter = EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&config.logging.log_level));
            init_tracing_subscriber(config, env_filter)
        }
        LogBackend::FastLog => init_fast_log(config),
    }
}

/// Shutdown observability
///
/// Flushes pending spans (relevant for OTEL).
pub fn shutdown() {
    #[cfg(feature = "otel")]
    {
        if let Some(provider) = OTEL_PROVIDER.get() {
            let _ = provider.shutdown();
        }
    }
}

fn init_tracing_subscriber(config: &Config, env_filter: EnvFilter) -> anyhow::Result<()> {
    // Console layer
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(config.logging.log_include_target)
        .with_span_events(FmtSpan::NONE);

    // Apply format
    let registry = tracing_subscriber::registry().with(env_filter);

    match config.logging.log_format {
        LogFormat::Pretty => {
            let registry = registry.with(
                fmt_layer
                    .pretty()
                    .with_file(config.logging.log_include_fileline)
                    .with_line_number(config.logging.log_include_fileline),
            );

            #[cfg(feature = "otel")]
            if config.features.feature_otel {
                let otel_layer = init_otel_layer(config)?;
                registry.with(otel_layer).try_init()?;
                return Ok(());
            }

            registry.try_init()?;
        }
        LogFormat::Compact => {
            let registry = registry.with(
                fmt_layer
                    .compact()
                    .with_ansi(false)
                    .with_file(config.logging.log_include_fileline)
                    .with_line_number(config.logging.log_include_fileline),
            );

            #[cfg(feature = "otel")]
            if config.features.feature_otel {
                let otel_layer = init_otel_layer(config)?;
                registry.with(otel_layer).try_init()?;
                return Ok(());
            }

            registry.try_init()?;
        }
        LogFormat::Json => {
            let registry = registry.with(
                fmt_layer
                    .json()
                    .with_file(config.logging.log_include_fileline)
                    .with_line_number(config.logging.log_include_fileline),
            );

            #[cfg(feature = "otel")]
            if config.features.feature_otel {
                let otel_layer = init_otel_layer(config)?;
                registry.with(otel_layer).try_init()?;
                return Ok(());
            }

            registry.try_init()?;
        }
    }

    Ok(())
}

fn init_fast_log(config: &Config) -> anyhow::Result<()> {
    #[cfg(feature = "fast-log")]
    {
        use fast_log::config::Config as FastLogConfig;

        if config.features.feature_otel {
            anyhow::bail!("LOG_BACKEND=fast_log is not compatible with FEATURE_OTEL=true");
        }

        if let Err(err) = fast_log::init(FastLogConfig::new().console()) {
            let message = err.to_string();
            if message.contains("logging system was already initialized") {
                anyhow::bail!("fast_log init failed because another logger is already set. Ensure init_tracing runs before any other logger initialization.");
            }
            return Err(err.into());
        }
        log::set_max_level(resolve_log_level(config));
        return Ok(());
    }

    #[cfg(not(feature = "fast-log"))]
    {
        let _ = config;
        anyhow::bail!("LOG_BACKEND=fast_log requires the \"fast-log\" feature on barrzen-axum-obs")
    }
}

#[cfg(feature = "fast-log")]
fn resolve_log_level(config: &Config) -> log::LevelFilter {
    let mut level = parse_log_level(&config.logging.log_level).unwrap_or(log::LevelFilter::Info);

    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        for directive in rust_log.split(',') {
            let directive = directive.trim();
            if directive.is_empty() {
                continue;
            }
            let level_str = directive
                .split_once('=')
                .map_or(directive, |(_, level)| level);
            if let Some(parsed) = parse_log_level(level_str) {
                level = level.max(parsed);
            }
        }
    }

    level
}

#[cfg(feature = "fast-log")]
fn parse_log_level(value: &str) -> Option<log::LevelFilter> {
    match value.trim().to_lowercase().as_str() {
        "off" => Some(log::LevelFilter::Off),
        "error" => Some(log::LevelFilter::Error),
        "warn" | "warning" => Some(log::LevelFilter::Warn),
        "info" => Some(log::LevelFilter::Info),
        "debug" => Some(log::LevelFilter::Debug),
        "trace" => Some(log::LevelFilter::Trace),
        _ => None,
    }
}

// OpenTelemetry Setup

#[cfg(feature = "otel")]
fn init_otel_layer<S>(config: &Config) -> anyhow::Result<impl Layer<S>>
where
    S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    use opentelemetry::{global, trace::TracerProvider as _};
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::{propagation::TraceContextPropagator, trace as sdktrace, Resource};
    use tracing_opentelemetry::OpenTelemetryLayer;

    // Set global propagator
    global::set_text_map_propagator(TraceContextPropagator::new());

    let app_name = &config.app.app_name;
    let otel_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    // OTEL 0.31: Use SpanExporter::builder().with_tonic()
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otel_endpoint)
        .build()?;

    // OTEL 0.31: Use Resource::builder()
    let resource = Resource::builder()
        .with_service_name(app_name.clone())
        .build();

    let provider = sdktrace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();
    
    // Set global provider
    global::set_tracer_provider(provider.clone());
    let _ = OTEL_PROVIDER.set(provider.clone());

    let tracer = provider.tracer("barrzen-axum");

    Ok(OpenTelemetryLayer::new(tracer))
}
