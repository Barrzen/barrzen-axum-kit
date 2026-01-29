//! Barrzen Axum Observability
//!
//! Handles tracing setup and OpenTelemetry integration.

use anyhow::Context;
use barrzen_axum_core::{Config, LogFormat};
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

/// Initialize tracing based on configuration
///
/// # Errors
/// Returns error if tracing subscriber setup fails.
pub fn init_tracing(config: &Config) -> anyhow::Result<()> {
    // Basic EnvFilter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.logging.log_level));

    // Console layer
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(config.logging.log_include_target)
        .with_file(config.logging.log_include_fileline)
        .with_line_number(config.logging.log_include_fileline)
        .with_span_events(FmtSpan::CLOSE);

    // Apply format
    let registry = tracing_subscriber::registry().with(env_filter);

    match config.logging.log_format {
        LogFormat::Pretty => {
            let registry = registry.with(fmt_layer.pretty());
            
            #[cfg(feature = "otel")]
            if config.features.feature_otel {
                let otel_layer = init_otel_layer(config)?;
                registry.with(otel_layer).try_init()?;
                return Ok(());
            }

            registry.try_init()?;
        }
        LogFormat::Json => {
            let registry = registry.with(fmt_layer.json());

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

/// Shutdown observability
///
/// Flushes pending spans (relevant for OTEL).
pub fn shutdown() {
    #[cfg(feature = "otel")]
    {
        opentelemetry::global::shutdown_tracer_provider();
    }
}

// OpenTelemetry Setup

#[cfg(feature = "otel")]
fn init_otel_layer<S>(config: &Config) -> anyhow::Result<impl Layer<S>>
where
    S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    use opentelemetry::{global, KeyValue, trace::TracerProvider as _};
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::{propagation::TraceContextPropagator, trace as sdktrace, Resource};
    use tracing_opentelemetry::OpenTelemetryLayer;

    // Set global propagator
    global::set_text_map_propagator(TraceContextPropagator::new());

    let app_name = &config.app.app_name;
    let otel_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otel_endpoint)
        .build()?;

    let provider = sdktrace::TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            app_name.clone(),
        )]))
        .build();
    
    // Set global provider so that users can access it without passing it around
    global::set_tracer_provider(provider.clone());

    let tracer = provider.tracer("barrzen-axum");

    Ok(OpenTelemetryLayer::new(tracer))
}
