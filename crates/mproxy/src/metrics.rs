use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{
    header::CONTENT_TYPE, server::conn::http1, service::service_fn, Method, Request, Response,
};
use opentelemetry::{
    global,
    sdk::{
        export::metrics::aggregation,
        metrics::{
            controllers::{self, BasicController},
            processors, selectors,
        },
        Resource,
    },
    KeyValue,
};
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info_span, instrument, Instrument, Subscriber};
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::Layer;

#[instrument(name = "metrics_request", skip_all)]
async fn serve_req(
    req: Request<hyper::body::Incoming>,
    state: Arc<AppState>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let response = match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => {
            let encoder = TextEncoder::new();
            let metric_families = state.exporter.registry().gather();
            let mut result = Vec::new();
            encoder.encode(&metric_families, &mut result).unwrap();

            Response::builder()
                .status(200)
                .header(CONTENT_TYPE, encoder.format_type())
                .body(full(result))
                .unwrap()
        }
        (&Method::GET, "/") => Response::builder()
            .status(200)
            .body(full("Hello World!"))
            .unwrap(),
        _ => Response::builder()
            .status(404)
            .body(full("Page not found"))
            .unwrap(),
    };

    Ok(response)
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

struct AppState {
    exporter: PrometheusExporter,
}

#[instrument(name = "prometheus_server", skip_all)]
pub async fn prometheus_server(controller: BasicController) -> Result<(), anyhow::Error> {
    let exporter = opentelemetry_prometheus::exporter(controller).init();

    let state = Arc::new(AppState { exporter });

    let listener = TcpListener::bind("127.0.0.1:18666").await?;

    loop {
        let (stream, _) = listener.accept().await?;

        let state = state.clone();

        tokio::task::spawn(
            async move {
                if let Err(err) = http1::Builder::new()
                    .preserve_header_case(true)
                    .title_case_headers(true)
                    .serve_connection(stream, service_fn(|req| serve_req(req, state.clone())))
                    .await
                {
                    error!("Failed to serve connection: {:?}", err);
                }
            }
            .instrument(info_span!("handle_connection")),
        );
    }
}

pub struct TelemetryDrop;

impl Drop for TelemetryDrop {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
    }
}

pub fn new_metrics_layer<S>() -> Result<
    (
        Vec<Box<dyn Layer<S> + Send + Sync + 'static>>,
        TelemetryDrop,
    ),
    anyhow::Error,
>
where
    S: Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span> + Sync + Send,
{
    let mut layers = Vec::new();

    {
        global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

        let tracer = opentelemetry_jaeger::new_agent_pipeline()
            .with_service_name("mproxy")
            .install_batch(opentelemetry::runtime::Tokio)?;

        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

        layers.push(telemetry.boxed());
    }

    {
        let controller = controllers::basic(
            processors::factory(
                selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]),
                aggregation::cumulative_temporality_selector(),
            )
            .with_memory(true),
        )
        .with_resource(Resource::new(vec![KeyValue::new("service.name", "mproxy")]))
        .build();

        tokio::spawn(prometheus_server(controller.clone()));
        layers.push(MetricsLayer::new(controller).boxed())
    }

    Ok((layers, TelemetryDrop))
}
