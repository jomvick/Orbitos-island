#[cfg(feature = "otlp")]
use std::time::SystemTime;
use std::sync::Arc;

#[cfg(feature = "otlp")]
use chrono::{DateTime, Utc};
use tokio::sync::broadcast;
use tracing::info;

use daemon_core::state::UniversalEvent;

#[allow(dead_code)]
pub struct OtlpExporter {
    endpoint: String,
}

impl OtlpExporter {
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }

    #[cfg(feature = "otlp")]
    pub async fn start(self, mut rx: broadcast::Receiver<Arc<UniversalEvent>>) {
        use opentelemetry::metrics::MeterProvider as _;
        use opentelemetry::trace::{Span, SpanKind, Tracer, TracerProvider as _};
        use opentelemetry::KeyValue;
        use opentelemetry_otlp::WithExportConfig;
        use opentelemetry_sdk::runtime;
        use opentelemetry_sdk::Resource;
        use tracing::{error, warn};

        let resource = Resource::new(vec![
            KeyValue::new("service.name", "agentosd"),
        ]);

        let span_exporter = match opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .with_endpoint(&self.endpoint)
            .build()
        {
            Ok(e) => e,
            Err(e) => {
                error!(error = %e, endpoint = %self.endpoint, "failed to build OTLP span exporter");
                return;
            }
        };

        let batch = opentelemetry_sdk::trace::BatchSpanProcessor::builder(
            span_exporter,
            runtime::Tokio,
        )
        .build();

        let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
            .with_span_processor(batch)
            .with_resource(resource.clone())
            .build();

        let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
            .with_resource(resource)
            .build();

        let tracer = tracer_provider.tracer("agentosd");
        let meter = meter_provider.meter("agentosd");
        opentelemetry::global::set_meter_provider(meter_provider);

        let event_counter = meter
            .u64_counter("agentos.events.total")
            .with_description("Total number of agent events")
            .build();

        let duration_histogram = meter
            .u64_histogram("agentos.duration.ms")
            .with_description("Event duration in milliseconds")
            .build();

        info!(endpoint = %self.endpoint, "OTLP exporter started");

        loop {
            match rx.recv().await {
                Ok(event) => {
                    let attrs = [
                        KeyValue::new("agent", event.agent.to_string()),
                        KeyValue::new("event_kind", event.event.to_string()),
                        KeyValue::new("session_id", event.session_id.clone()),
                    ];

                    event_counter.add(1, &attrs);

                    if let Some(dur) = event.duration_ms {
                        duration_histogram.record(dur, &attrs);
                    }

                    let span_name = format!("{}/{}", event.agent, event.event);
                    let mut span = tracer
                        .span_builder(span_name)
                        .with_kind(SpanKind::Consumer)
                        .with_start_time(otlp_time(event.timestamp))
                        .start(&tracer);

                    span.set_attribute(KeyValue::new("agentos.agent", event.agent.to_string()));
                    span.set_attribute(KeyValue::new("agentos.event_kind", event.event.to_string()));
                    span.set_attribute(KeyValue::new("agentos.session_id", event.session_id.clone()));

                    if let Some(ref v) = event.cwd {
                        span.set_attribute(KeyValue::new("agentos.cwd", v.clone()));
                    }
                    if let Some(ref v) = event.branch {
                        span.set_attribute(KeyValue::new("agentos.branch", v.clone()));
                    }
                    if let Some(ref v) = event.model {
                        span.set_attribute(KeyValue::new("agentos.model", v.clone()));
                    }
                    if let Some(v) = event.tokens_input {
                        span.set_attribute(KeyValue::new("agentos.tokens_input", v as i64));
                    }
                    if let Some(v) = event.tokens_output {
                        span.set_attribute(KeyValue::new("agentos.tokens_output", v as i64));
                    }
                    if let Some(v) = event.duration_ms {
                        span.set_attribute(KeyValue::new("agentos.duration_ms", v as i64));
                    }
                    if let Some(ref v) = event.terminal {
                        span.set_attribute(KeyValue::new("agentos.terminal", v.clone()));
                    }
                    if let Some(ref v) = event.error {
                        span.set_attribute(KeyValue::new("agentos.error", v.clone()));
                        span.set_status(opentelemetry::trace::Status::error(v.clone()));
                    } else {
                        span.set_status(opentelemetry::trace::Status::Ok);
                    }

                    span.end();
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!(count = %n, "OTLP exporter lagged behind");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }

        info!("OTLP exporter stopped");
        let _ = opentelemetry::global::shutdown_tracer_provider();
    }

    #[cfg(not(feature = "otlp"))]
    pub async fn start(self, mut rx: broadcast::Receiver<Arc<UniversalEvent>>) {
        info!("OTLP exporter: feature disabled (recompile with 'otlp' feature)");
        loop {
            match rx.recv().await {
                Ok(_) => {}
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    }
}

#[cfg(feature = "otlp")]
fn otlp_time(dt: DateTime<Utc>) -> SystemTime {
    SystemTime::UNIX_EPOCH
        + std::time::Duration::from_secs(dt.timestamp() as u64)
        + std::time::Duration::from_nanos(dt.timestamp_subsec_nanos() as u64)
}
