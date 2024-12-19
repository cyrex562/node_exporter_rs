use flate2::write::GzEncoder;
use flate2::Compression;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use prometheus::{self, Counter, CounterVec, Encoder, Gauge, Opts, Registry, TextEncoder};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::sync::Semaphore;
use tokio::time::timeout;
use zstd::stream::write::Encoder as ZstdEncoder;

const CONTENT_TYPE_HEADER: &str = "Content-Type";
const CONTENT_ENCODING_HEADER: &str = "Content-Encoding";
const ACCEPT_ENCODING_HEADER: &str = "Accept-Encoding";
const PROCESS_START_TIME_HEADER: &str = "Process-Start-Time-Unix";

#[derive(Clone, Copy)]
enum Compression {
    Identity,
    Gzip,
    Zstd,
}

impl Compression {
    fn as_str(&self) -> &'static str {
        match self {
            Compression::Identity => "identity",
            Compression::Gzip => "gzip",
            Compression::Zstd => "zstd",
        }
    }
}

struct HandlerOpts {
    error_log: Option<Arc<dyn Logger>>,
    error_handling: HandlerErrorHandling,
    registry: Option<Registry>,
    disable_compression: bool,
    offered_compressions: Vec<Compression>,
    max_requests_in_flight: usize,
    timeout: Duration,
    enable_open_metrics: bool,
    enable_open_metrics_text_created_samples: bool,
    process_start_time: Option<SystemTime>,
}

#[derive(Clone, Copy)]
enum HandlerErrorHandling {
    HTTPErrorOnError,
    ContinueOnError,
    PanicOnError,
}

trait Logger: Send + Sync {
    fn println(&self, v: &str);
}

async fn handler(opts: HandlerOpts) -> impl Fn(Request<Body>) -> Response<Body> + Clone {
    let in_flight_sem = if opts.max_requests_in_flight > 0 {
        Some(Arc::new(Semaphore::new(opts.max_requests_in_flight)))
    } else {
        None
    };

    let err_cnt = CounterVec::new(
        Opts::new(
            "promhttp_metric_handler_errors_total",
            "Total number of internal errors encountered by the promhttp metric handler.",
        ),
        &["cause"],
    )
    .unwrap();

    if let Some(registry) = &opts.registry {
        registry.register(Box::new(err_cnt.clone())).unwrap();
    }

    let compressions: Vec<&str> = if opts.disable_compression {
        vec!["identity"]
    } else {
        opts.offered_compressions
            .iter()
            .map(|c| c.as_str())
            .collect()
    };

    move |req: Request<Body>| {
        let opts = opts.clone();
        let in_flight_sem = in_flight_sem.clone();
        let err_cnt = err_cnt.clone();
        let compressions = compressions.clone();

        async move {
            if let Some(sem) = in_flight_sem {
                if sem.try_acquire().is_err() {
                    return Response::builder()
                        .status(503)
                        .body(Body::from(format!(
                            "Limit of concurrent requests reached ({}), try again later.",
                            opts.max_requests_in_flight
                        )))
                        .unwrap();
                }
            }

            let gatherer = prometheus::default_registry().gather();
            let encoder = TextEncoder::new();
            let mut buffer = Vec::new();
            if let Err(e) = encoder.encode(&gatherer, &mut buffer) {
                if let Some(logger) = &opts.error_log {
                    logger.println(&format!("error gathering metrics: {}", e));
                }
                err_cnt.with_label_values(&["gathering"]).inc();
                match opts.error_handling {
                    HandlerErrorHandling::PanicOnError => panic!("{}", e),
                    HandlerErrorHandling::HTTPErrorOnError => {
                        return Response::builder()
                            .status(500)
                            .body(Body::from(format!(
                                "An error has occurred while serving metrics:\n\n{}",
                                e
                            )))
                            .unwrap();
                    }
                    HandlerErrorHandling::ContinueOnError => {}
                }
            }

            let mut response = Response::builder()
                .header(CONTENT_TYPE_HEADER, encoder.format_type())
                .body(Body::from(buffer))
                .unwrap();

            if let Some(start_time) = opts.process_start_time {
                response.headers_mut().insert(
                    PROCESS_START_TIME_HEADER,
                    start_time
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        .to_string()
                        .parse()
                        .unwrap(),
                );
            }

            if !opts.disable_compression {
                let encoding = req
                    .headers()
                    .get(ACCEPT_ENCODING_HEADER)
                    .and_then(|val| val.to_str().ok())
                    .unwrap_or("");
                if encoding.contains("gzip") {
                    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                    encoder.write_all(response.body().as_ref()).unwrap();
                    let compressed_body = encoder.finish().unwrap();
                    response
                        .headers_mut()
                        .insert(CONTENT_ENCODING_HEADER, "gzip".parse().unwrap());
                    *response.body_mut() = Body::from(compressed_body);
                } else if encoding.contains("zstd") {
                    let mut encoder = ZstdEncoder::new(Vec::new(), 0).unwrap();
                    encoder.write_all(response.body().as_ref()).unwrap();
                    let compressed_body = encoder.finish().unwrap();
                    response
                        .headers_mut()
                        .insert(CONTENT_ENCODING_HEADER, "zstd".parse().unwrap());
                    *response.body_mut() = Body::from(compressed_body);
                }
            }

            response
        }
    }
}

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use prometheus::{register_counter_vec, register_gauge, CounterVec, Gauge, Opts, Registry};

fn instrument_metric_handler(
    reg: &Registry,
    handler: impl Fn(Request<Body>) -> Response<Body> + Clone + Send + 'static,
) -> impl Fn(Request<Body>) -> Response<Body> + Clone + Send + 'static {
    let cnt = register_counter_vec!(
        Opts::new(
            "promhttp_metric_handler_requests_total",
            "Total number of scrapes by HTTP status code."
        ),
        &["code"]
    )
    .unwrap();

    // Initialize the most likely HTTP status codes.
    cnt.with_label_values(&["200"]);
    cnt.with_label_values(&["500"]);
    cnt.with_label_values(&["503"]);

    if let Err(err) = reg.register(Box::new(cnt.clone())) {
        if let Some(are) = err.downcast_ref::<prometheus::AlreadyRegisteredError>() {
            cnt = are
                .existing_collector()
                .clone()
                .downcast::<CounterVec>()
                .unwrap();
        } else {
            panic!("{}", err);
        }
    }

    let gge = register_gauge!(Opts::new(
        "promhttp_metric_handler_requests_in_flight",
        "Current number of scrapes being served."
    ))
    .unwrap();

    if let Err(err) = reg.register(Box::new(gge.clone())) {
        if let Some(are) = err.downcast_ref::<prometheus::AlreadyRegisteredError>() {
            gge = are
                .existing_collector()
                .clone()
                .downcast::<Gauge>()
                .unwrap();
        } else {
            panic!("{}", err);
        }
    }

    instrument_handler_counter(cnt, instrument_handler_in_flight(gge, handler))
}

enum HandlerErrorHandling {
    HTTPErrorOnError,
    ContinueOnError,
    PanicOnError,
}

trait Logger: Send + Sync {
    fn println(&self, v: &str);
}

use prometheus::Registry;
use std::time::Duration;

struct HandlerOpts {
    // ErrorLog specifies an optional Logger for errors collecting and serving metrics.
    // If None, errors are not logged at all.
    error_log: Option<Box<dyn Logger>>,

    // ErrorHandling defines how errors are handled.
    error_handling: HandlerErrorHandling,

    // If registry is not None, it is used to register a metric "promhttp_metric_handler_errors_total", partitioned by "cause".
    registry: Option<Registry>,

    // DisableCompression disables the response encoding (compression) and encoding negotiation.
    disable_compression: bool,

    // OfferedCompressions is a set of encodings (compressions) handler will try to offer when negotiating with the client.
    offered_compressions: Vec<Compression>,

    // The number of concurrent HTTP requests is limited to max_requests_in_flight.
    max_requests_in_flight: i32,

    // If handling a request takes longer than timeout, it is responded to with 503 ServiceUnavailable and a suitable message.
    timeout: Duration,

    // If true, the experimental OpenMetrics encoding is added to the possible options during content negotiation.
    enable_open_metrics: bool,

    // EnableOpenMetricsTextCreatedSamples specifies if this handler should add extra, synthetic Created Timestamps for counters, histograms, and summaries.
    enable_open_metrics_text_created_samples: bool,

    // ProcessStartTime allows setting process start time value that will be exposed with "Process-Start-Time-Unix" response header along with the metrics payload.
    process_start_time: Option<Duration>,
}

use hyper::{Body, Response, StatusCode};

fn http_error(rsp: &mut Response<Body>, err: &str) {
    rsp.headers_mut().remove(hyper::header::CONTENT_ENCODING);
    *rsp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    *rsp.body_mut() = Body::from(format!(
        "An error has occurred while serving metrics:\n\n{}",
        err
    ));
}

use flate2::write::GzEncoder;
use flate2::Compression;
use hyper::header::HeaderValue;
use hyper::Request;
use std::io::{self, Write};
use zstd::stream::write::Encoder as ZstdEncoder;

fn negotiate_encoding_writer(
    req: &Request<hyper::Body>,
    rw: Box<dyn Write>,
    compressions: &[&str],
) -> Result<(Box<dyn Write>, String, Box<dyn FnOnce()>), io::Error> {
    if compressions.is_empty() {
        return Ok((rw, "identity".to_string(), Box::new(|| {})));
    }

    let selected = httputil::negotiate_content_encoding(req, compressions);

    match selected.as_str() {
        "zstd" => {
            let mut z = ZstdEncoder::new(rw, 0)?;
            let writer = Box::new(z) as Box<dyn Write>;
            let close_writer = Box::new(move || {
                let _ = z.finish();
            });
            Ok((writer, selected, close_writer))
        }
        "gzip" => {
            let mut gz = GzEncoder::new(rw, Compression::default());
            let writer = Box::new(gz) as Box<dyn Write>;
            let close_writer = Box::new(move || {
                let _ = gz.finish();
            });
            Ok((writer, selected, close_writer))
        }
        "identity" => Ok((rw, selected, Box::new(|| {}))),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "content compression format not recognized: {}. Valid formats are: {:?}",
                selected, compressions
            ),
        )),
    }
}
