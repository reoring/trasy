use opentelemetry::KeyValue;
use opentelemetry_otlp::SpanExporterBuilder;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace;
use opentelemetry_sdk::trace::Tracer;
use opentelemetry_sdk::Resource;
use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt;
use tracing_error::SpanTrace;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::Registry;

#[derive(Debug)]
pub struct TrasyError<T> {
    context: SpanTrace,
    backtrace: Option<Backtrace>,
    inner: T,
}

impl<T> TrasyError<T> {
    pub fn new(inner: T) -> Self {
        Self {
            context: SpanTrace::capture(),
            backtrace: None,
            inner,
        }
    }

    pub fn with_backtrace(mut self, backtrace: Backtrace) -> Self {
        self.backtrace = Some(backtrace);
        self
    }
}

impl<T: fmt::Debug + fmt::Display> fmt::Display for TrasyError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}\nContext: {}\n", self.inner, self.context)?;
        if let Some(ref backtrace) = self.backtrace {
            write!(f, "Backtrace: {:?}\n", backtrace)?;
        }
        Ok(())
    }
}

impl<T: fmt::Debug + fmt::Display + Error + AsRef<dyn Error>> Error for TrasyError<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.inner.as_ref())
    }
}

#[macro_export]
macro_rules! error {
    ($e:expr) => {
        TrasyError::new($e).with_backtrace(std::backtrace::Backtrace::capture())
    };
}

#[macro_export]
macro_rules! bail {
    ($e:expr) => {
        Err(TrasyError::new($e).with_backtrace(std::backtrace::Backtrace::capture()))
    };
}

struct TelemetryConfig {
    service_name: String,
    #[allow(dead_code)]
    endpoint: String,
    use_batch: bool, // Determine whether to use batch or simple span processing
    oltp_exporter: Option<SpanExporterBuilder>,
}

impl TelemetryConfig {
    #[allow(dead_code)]
    pub fn with_oltp_exporter<B: Into<SpanExporterBuilder>>(mut self, exporter: B) -> Self {
        self.oltp_exporter = Some(exporter.into());
        self
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        let endpoint = "http://localhost:4318";

        let otlp_exporter = opentelemetry_otlp::new_exporter()
            .http()
            .with_endpoint(endpoint);

        Self {
            service_name: "default-service".to_string(),
            endpoint: endpoint.to_string(),
            use_batch: true,
            oltp_exporter: Some(otlp_exporter.into()),
        }
    }
}

#[allow(dead_code)]
async fn setup_opentelemetry(
    config: TelemetryConfig,
) -> Result<OpenTelemetryLayer<Registry, Tracer>, TrasyError<std::io::Error>> {
    let Some(exporter) = config.oltp_exporter else {
        return Err(TrasyError::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "oltp_exporter is None",
        )));
    };

    let service_name = config.service_name;

    let builder = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            trace::config().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                service_name,
            )])),
        );

    let tracer = if config.use_batch {
        builder.install_batch(opentelemetry_sdk::runtime::Tokio)
    } else {
        builder.install_simple()
    }
    .expect("Error initializing OpenTelemetry exporter");

    let opentelemetry: OpenTelemetryLayer<Registry, Tracer> =
        tracing_opentelemetry::layer().with_tracer(tracer);
    Ok(opentelemetry)
}
