use hyper::client::connect::Connect;
use hyper::client::Client;
use hyper::Request;
use hyper::Response;
use hyper::body::HttpBody;
use prometheus::{CounterVec, Gauge, HistogramVec, Opts, Registry};
use std::sync::Arc;
use std::time::Instant;
use tokio::time::Duration;

pub type RoundTripper = Arc<dyn Fn(Request<hyper::Body>) -> hyper::client::ResponseFuture + Send + Sync>;

pub fn instrument_round_tripper_in_flight(gauge: Gauge, next: RoundTripper) -> RoundTripper {
    Arc::new(move |req: Request<hyper::Body>| {
        let gauge = gauge.clone();
        gauge.inc();
        let fut = next(req);
        async move {
            let res = fut.await;
            gauge.dec();
            res
        }
    })
}

pub fn instrument_round_tripper_counter(counter: CounterVec, next: RoundTripper) -> RoundTripper {
    Arc::new(move |req: Request<hyper::Body>| {
        let counter = counter.clone();
        let method = req.method().to_string();
        let fut = next(req);
        async move {
            let res = fut.await;
            if let Ok(ref response) = res {
                let status = response.status().as_u16().to_string();
                counter.with_label_values(&[&status, &method]).inc();
            }
            res
        }
    })
}

pub fn instrument_round_tripper_duration(histogram: HistogramVec, next: RoundTripper) -> RoundTripper {
    Arc::new(move |req: Request<hyper::Body>| {
        let histogram = histogram.clone();
        let method = req.method().to_string();
        let start = Instant::now();
        let fut = next(req);
        async move {
            let res = fut.await;
            if let Ok(ref response) = res {
                let status = response.status().as_u16().to_string();
                let duration = start.elapsed().as_secs_f64();
                histogram.with_label_values(&[&status, &method]).observe(duration);
            }
            res
        }
    })
}

pub struct InstrumentTrace {
    pub got_conn: Option<Box<dyn Fn(f64) + Send + Sync>>,
    pub put_idle_conn: Option<Box<dyn Fn(f64) + Send + Sync>>,
    pub got_first_response_byte: Option<Box<dyn Fn(f64) + Send + Sync>>,
    pub got_100_continue: Option<Box<dyn Fn(f64) + Send + Sync>>,
    pub dns_start: Option<Box<dyn Fn(f64) + Send + Sync>>,
    pub dns_done: Option<Box<dyn Fn(f64) + Send + Sync>>,
    pub connect_start: Option<Box<dyn Fn(f64) + Send + Sync>>,
    pub connect_done: Option<Box<dyn Fn(f64) + Send + Sync>>,
    pub tls_handshake_start: Option<Box<dyn Fn(f64) + Send + Sync>>,
    pub tls_handshake_done: Option<Box<dyn Fn(f64) + Send + Sync>>,
    pub wrote_headers: Option<Box<dyn Fn(f64) + Send + Sync>>,
    pub wait_100_continue: Option<Box<dyn Fn(f64) + Send + Sync>>,
    pub wrote_request: Option<Box<dyn Fn(f64) + Send + Sync>>,
}

pub fn instrument_round_tripper_trace(trace: InstrumentTrace, next: RoundTripper) -> RoundTripper {
    Arc::new(move |req: Request<hyper::Body>| {
        let trace = trace.clone();
        let start = Instant::now();
        let fut = next(req);
        async move {
            let res = fut.await;
            if let Some(ref got_conn) = trace.got_conn {
                got_conn(start.elapsed().as_secs_f64());
            }
            if let Some(ref put_idle_conn) = trace.put_idle_conn {
                put_idle_conn(start.elapsed().as_secs_f64());
            }
            if let Some(ref dns_start) = trace.dns_start {
                dns_start(start.elapsed().as_secs_f64());
            }
            if let Some(ref dns_done) = trace.dns_done {
                dns_done(start.elapsed().as_secs_f64());
            }
            if let Some(ref connect_start) = trace.connect_start {
                connect_start(start.elapsed().as_secs_f64());
            }
            if let Some(ref connect_done) = trace.connect_done {
                connect_done(start.elapsed().as_secs_f64());
            }
            if let Some(ref got_first_response_byte) = trace.got_first_response_byte {
                got_first_response_byte(start.elapsed().as_secs_f64());
            }
            if let Some(ref got_100_continue) = trace.got_100_continue {
                got_100_continue(start.elapsed().as_secs_f64());
            }
            if let Some(ref tls_handshake_start) = trace.tls_handshake_start {
                tls_handshake_start(start.elapsed().as_secs_f64());
            }
            if let Some(ref tls_handshake_done) = trace.tls_handshake_done {
                tls_handshake_done(start.elapsed().as_secs_f64());
            }
            if let Some(ref wrote_headers) = trace.wrote_headers {
                wrote_headers(start.elapsed().as_secs_f64());
            }
            if let Some(ref wait_100_continue) = trace.wait_100_continue {
                wait_100_continue(start.elapsed().as_secs_f64());
            }
            if let Some(ref wrote_request) = trace.wrote_request {
                wrote_request(start.elapsed().as_secs_f64());
            }
            res
        }
    })
}