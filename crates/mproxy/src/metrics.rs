use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{
    header::CONTENT_TYPE, server::conn::http1, service::service_fn, Method, Request, Response,
};
use opentelemetry::sdk::metrics::controllers::BasicController;
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info_span, instrument, Instrument};

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

// fn empty() -> BoxBody<Bytes, hyper::Error> {
//     Empty::<Bytes>::new()
//         .map_err(|never| match never {})
//         .boxed()
// }

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
