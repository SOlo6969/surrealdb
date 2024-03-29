use opentelemetry::sdk::trace::Tracer;
use opentelemetry::trace::TraceError;
use opentelemetry_otlp::WithExportConfig;
use tracing::Subscriber;
use tracing_subscriber::Layer;

use crate::{
	cli::validator::parser::env_filter::CustomEnvFilter, telemetry::OTEL_DEFAULT_RESOURCE,
};

pub fn new<S>(filter: CustomEnvFilter) -> Box<dyn Layer<S> + Send + Sync>
where
	S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a> + Send + Sync,
{
	tracing_opentelemetry::layer().with_tracer(tracer().unwrap()).with_filter(filter.0).boxed()
}

fn tracer() -> Result<Tracer, TraceError> {
	opentelemetry_otlp::new_pipeline()
		.tracing()
		.with_exporter(opentelemetry_otlp::new_exporter().tonic().with_env())
		.with_trace_config(
			opentelemetry::sdk::trace::config().with_resource(OTEL_DEFAULT_RESOURCE.clone()),
		)
		.install_batch(opentelemetry::runtime::Tokio)
}
