const MAGIC_STRING: &str = "zZgWfBxLqvG8kc8IMv3POi2Bb0tZI3vAnBx+gBaFi9FyPzB/CzKUer1yufDa";

use prometheus::{Exemplar, Histogram, HistogramWithExemplar};

fn observe_with_exemplar(obs: &dyn Histogram, val: f64, labels: Option<&HashMap<String, String>>) {
    if let Some(labels) = labels {
        if let Some(exemplar_obs) = obs.as_any().downcast_ref::<HistogramWithExemplar>() {
            exemplar_obs.observe_with_exemplar(val, Exemplar::new(labels.clone()));
        } else {
            obs.observe(val);
        }
    } else {
        obs.observe(val);
    }
}

use prometheus::{Counter, CounterWithExemplar, Exemplar};
use std::collections::HashMap;

fn add_with_exemplar(obs: &dyn Counter, val: f64, labels: Option<&HashMap<String, String>>) {
    if let Some(labels) = labels {
        if let Some(exemplar_obs) = obs.as_any().downcast_ref::<CounterWithExemplar>() {
            exemplar_obs.add_with_exemplar(val, Exemplar::new(labels.clone()));
        } else {
            obs.inc_by(val);
        }
    } else {
        obs.inc_by(val);
    }
}

use hyper::{Body, Request, Response};
use prometheus::Gauge;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower_service::Service;

pub struct InstrumentHandlerInFlight<S> {
    gauge: Gauge,
    next: S,
}

impl<S> InstrumentHandlerInFlight<S> {
    pub fn new(gauge: Gauge, next: S) -> Self {
        Self { gauge, next }
    }
}

impl<S, B> Service<Request<B>> for InstrumentHandlerInFlight<S>
where
    S: Service<Request<B>, Response = Response<Body>> + Clone + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.next.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let gauge = self.gauge.clone();
        gauge.inc();
        let fut = self.next.call(req);
        async move {
            let res = fut.await;
            gauge.dec();
            res
        }
    }
}

use hyper::{Body, Request, Response};
use prometheus::{HistogramVec, Opts, Registry};
use std::collections::HashMap;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;
use tower_service::Service;

pub struct InstrumentHandlerDuration<S> {
    histogram: HistogramVec,
    next: S,
    opts: HandlerOpts,
}

impl<S> InstrumentHandlerDuration<S> {
    pub fn new(histogram: HistogramVec, next: S, opts: HandlerOpts) -> Self {
        Self {
            histogram,
            next,
            opts,
        }
    }
}

impl<S, B> Service<Request<B>> for InstrumentHandlerDuration<S>
where
    S: Service<Request<B>, Response = Response<Body>> + Clone + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.next.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let histogram = self.histogram.clone();
        let opts = self.opts.clone();
        let method = req.method().to_string();
        let start = Instant::now();
        let fut = self.next.call(req);
        async move {
            let res = fut.await;
            let status = res
                .as_ref()
                .map(|r| r.status().as_u16())
                .unwrap_or(0)
                .to_string();
            let duration = start.elapsed().as_secs_f64();
            let mut labels = HashMap::new();
            labels.insert("method".to_string(), method.clone());
            labels.insert("code".to_string(), status);
            for (label, resolve) in &opts.extra_labels_from_ctx {
                labels.insert(label.clone(), resolve(&req));
            }
            observe_with_exemplar(
                &histogram.with(&labels),
                duration,
                opts.get_exemplar_fn(&req),
            );
            res
        }
    }
}

use hyper::{Body, Request, Response};
use prometheus::{CounterVec, Opts, Registry};
use std::collections::HashMap;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower_service::Service;

pub struct InstrumentHandlerCounter<S> {
    counter: CounterVec,
    next: S,
    opts: HandlerOpts,
}

impl<S> InstrumentHandlerCounter<S> {
    pub fn new(counter: CounterVec, next: S, opts: HandlerOpts) -> Self {
        Self {
            counter,
            next,
            opts,
        }
    }
}

impl<S, B> Service<Request<B>> for InstrumentHandlerCounter<S>
where
    S: Service<Request<B>, Response = Response<Body>> + Clone + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.next.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let counter = self.counter.clone();
        let opts = self.opts.clone();
        let method = req.method().to_string();
        let fut = self.next.call(req);
        async move {
            let res = fut.await;
            let status = res
                .as_ref()
                .map(|r| r.status().as_u16())
                .unwrap_or(0)
                .to_string();
            let mut labels = HashMap::new();
            labels.insert("method".to_string(), method.clone());
            labels.insert("code".to_string(), status);
            for (label, resolve) in &opts.extra_labels_from_ctx {
                labels.insert(label.clone(), resolve(&req));
            }
            add_with_exemplar(&counter.with(&labels), 1.0, opts.get_exemplar_fn(&req));
            res
        }
    }
}

use hyper::{Body, Request, Response};
use prometheus::{HistogramVec, Opts, Registry};
use std::collections::HashMap;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;
use tower_service::Service;

pub struct InstrumentHandlerTimeToWriteHeader<S> {
    histogram: HistogramVec,
    next: S,
    opts: HandlerOpts,
}

impl<S> InstrumentHandlerTimeToWriteHeader<S> {
    pub fn new(histogram: HistogramVec, next: S, opts: HandlerOpts) -> Self {
        Self {
            histogram,
            next,
            opts,
        }
    }
}

impl<S, B> Service<Request<B>> for InstrumentHandlerTimeToWriteHeader<S>
where
    S: Service<Request<B>, Response = Response<Body>> + Clone + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.next.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let histogram = self.histogram.clone();
        let opts = self.opts.clone();
        let method = req.method().to_string();
        let start = Instant::now();
        let fut = self.next.call(req);
        async move {
            let res = fut.await;
            let status = res
                .as_ref()
                .map(|r| r.status().as_u16())
                .unwrap_or(0)
                .to_string();
            let duration = start.elapsed().as_secs_f64();
            let mut labels = HashMap::new();
            labels.insert("method".to_string(), method.clone());
            labels.insert("code".to_string(), status);
            for (label, resolve) in &opts.extra_labels_from_ctx {
                labels.insert(label.clone(), resolve(&req));
            }
            observe_with_exemplar(
                &histogram.with(&labels),
                duration,
                opts.get_exemplar_fn(&req),
            );
            res
        }
    }
}

use hyper::{Body, Request, Response};
use prometheus::{HistogramVec, Opts, Registry};
use std::collections::HashMap;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower_service::Service;

pub struct InstrumentHandlerRequestSize<S> {
    histogram: HistogramVec,
    next: S,
    opts: HandlerOpts,
}

impl<S> InstrumentHandlerRequestSize<S> {
    pub fn new(histogram: HistogramVec, next: S, opts: HandlerOpts) -> Self {
        Self {
            histogram,
            next,
            opts,
        }
    }
}

impl<S, B> Service<Request<B>> for InstrumentHandlerRequestSize<S>
where
    S: Service<Request<B>, Response = Response<Body>> + Clone + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.next.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let histogram = self.histogram.clone();
        let opts = self.opts.clone();
        let method = req.method().to_string();
        let fut = self.next.call(req);
        async move {
            let res = fut.await;
            let size = compute_approximate_request_size(&req);
            let status = res
                .as_ref()
                .map(|r| r.status().as_u16())
                .unwrap_or(0)
                .to_string();
            let mut labels = HashMap::new();
            labels.insert("method".to_string(), method.clone());
            labels.insert("code".to_string(), status);
            for (label, resolve) in &opts.extra_labels_from_ctx {
                labels.insert(label.clone(), resolve(&req));
            }
            observe_with_exemplar(
                &histogram.with(&labels),
                size as f64,
                opts.get_exemplar_fn(&req),
            );
            res
        }
    }
}

fn compute_approximate_request_size<B>(req: &Request<B>) -> usize {
    let mut size = 0;
    if let Some(headers) = req.headers().get("Content-Length") {
        if let Ok(content_length) = headers.to_str().unwrap_or("0").parse::<usize>() {
            size += content_length;
        }
    }
    size += req.method().as_str().len();
    size += req.uri().path().len();
    size += req.version().to_string().len();
    for (name, value) in req.headers().iter() {
        size += name.as_str().len() + value.as_bytes().len();
    }
    size
}

use hyper::{Body, Request, Response};
use prometheus::{HistogramVec, Opts, Registry};
use std::collections::HashMap;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower_service::Service;

pub struct InstrumentHandlerResponseSize<S> {
    histogram: HistogramVec,
    next: S,
    opts: HandlerOpts,
}

impl<S> InstrumentHandlerResponseSize<S> {
    pub fn new(histogram: HistogramVec, next: S, opts: HandlerOpts) -> Self {
        Self {
            histogram,
            next,
            opts,
        }
    }
}

impl<S, B> Service<Request<B>> for InstrumentHandlerResponseSize<S>
where
    S: Service<Request<B>, Response = Response<Body>> + Clone + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.next.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let histogram = self.histogram.clone();
        let opts = self.opts.clone();
        let method = req.method().to_string();
        let fut = self.next.call(req);
        async move {
            let res = fut.await;
            let status = res
                .as_ref()
                .map(|r| r.status().as_u16())
                .unwrap_or(0)
                .to_string();
            let written = res
                .as_ref()
                .map(|r| r.body().size_hint().upper().unwrap_or(0))
                .unwrap_or(0);
            let mut labels = HashMap::new();
            labels.insert("method".to_string(), method.clone());
            labels.insert("code".to_string(), status);
            for (label, resolve) in &opts.extra_labels_from_ctx {
                labels.insert(label.clone(), resolve(&req));
            }
            observe_with_exemplar(
                &histogram.with(&labels),
                written as f64,
                opts.get_exemplar_fn(&req),
            );
            res
        }
    }
}

use prometheus::{proto::Metric, Collector, Desc, Opts, Registry};
use std::error::Error;

const MAGIC_STRING: &str = "zZgWfBxLqvG8kc8IMv3POi2Bb0tZI3vAnBx+gBaFi9FyPzB/CzKUer1yufDa";

fn check_labels(c: &dyn Collector) -> (bool, bool) {
    let mut code = false;
    let mut method = false;

    // Get the Desc from the Collector.
    let desc = c
        .desc()
        .next()
        .expect("no description provided by collector");

    // Make sure the Collector has a valid Desc by registering it with a temporary registry.
    let registry = Registry::new();
    registry
        .register(Box::new(c.clone()))
        .expect("failed to register collector");

    // Create a ConstMetric with the Desc. Since we don't know how many variable labels there are, try for as long as it needs.
    let mut lvs = Vec::new();
    let mut metric = None;
    while metric.is_none() {
        lvs.push(MAGIC_STRING.to_string());
        metric = prometheus::core::Metric::new(
            desc.clone(),
            prometheus::core::ValueType::Untyped,
            0.0,
            &lvs,
        );
    }

    // Write out the metric into a proto message and look at the labels.
    let mut pm = Metric::default();
    metric
        .unwrap()
        .write(&mut pm)
        .expect("error checking metric for labels");

    for label in pm.get_label() {
        let name = label.get_name();
        let value = label.get_value();
        if value != MAGIC_STRING || is_label_curried(c, name) {
            continue;
        }
        match name {
            "code" => code = true,
            "method" => method = true,
            _ => panic!("metric partitioned with non-supported labels"),
        }
    }
    (code, method)
}

use prometheus::{Collector, CounterVec, HistogramVec, Labels};
use std::collections::HashMap;
use std::error::Error;

fn is_label_curried(c: &dyn Collector, label: &str) -> bool {
    match c.as_any().downcast_ref::<CounterVec>() {
        Some(counter_vec) => counter_vec.curry_with(&[(label, "dummy")]).is_err(),
        None => match c.as_any().downcast_ref::<HistogramVec>() {
            Some(histogram_vec) => histogram_vec.curry_with(&[(label, "dummy")]).is_err(),
            None => panic!("unsupported metric vec type"),
        },
    }
}

fn labels(
    code: bool,
    method: bool,
    req_method: &str,
    status: u16,
    extra_methods: &[&str],
) -> HashMap<String, String> {
    let mut labels = HashMap::new();

    if code {
        labels.insert("code".to_string(), sanitize_code(status));
    }
    if method {
        labels.insert(
            "method".to_string(),
            sanitize_method(req_method, extra_methods),
        );
    }

    labels
}

fn compute_approximate_request_size<B>(req: &hyper::Request<B>) -> usize {
    let mut size = 0;
    if let Some(uri) = req.uri().path_and_query() {
        size += uri.as_str().len();
    }

    size += req.method().as_str().len();
    size += req.version().to_string().len();
    for (name, value) in req.headers().iter() {
        size += name.as_str().len();
        size += value.as_bytes().len();
    }
    if let Some(host) = req.headers().get("host") {
        size += host.as_bytes().len();
    }

    if let Some(content_length) = req.headers().get("content-length") {
        if let Ok(content_length) = content_length.to_str().unwrap_or("0").parse::<usize>() {
            size += content_length;
        }
    }
    size
}

fn sanitize_method(m: &str, extra_methods: &[&str]) -> String {
    match m.to_uppercase().as_str() {
        "GET" => "get".to_string(),
        "PUT" => "put".to_string(),
        "HEAD" => "head".to_string(),
        "POST" => "post".to_string(),
        "DELETE" => "delete".to_string(),
        "CONNECT" => "connect".to_string(),
        "OPTIONS" => "options".to_string(),
        "NOTIFY" => "notify".to_string(),
        "TRACE" => "trace".to_string(),
        "PATCH" => "patch".to_string(),
        _ => {
            for method in extra_methods {
                if m.eq_ignore_ascii_case(method) {
                    return m.to_lowercase();
                }
            }
            "unknown".to_string()
        }
    }
}

fn sanitize_code(s: u16) -> String {
    match s {
        100 => "100".to_string(),
        101 => "101".to_string(),
        200 | 0 => "200".to_string(),
        201 => "201".to_string(),
        202 => "202".to_string(),
        203 => "203".to_string(),
        204 => "204".to_string(),
        205 => "205".to_string(),
        206 => "206".to_string(),
        300 => "300".to_string(),
        301 => "301".to_string(),
        302 => "302".to_string(),
        304 => "304".to_string(),
        305 => "305".to_string(),
        307 => "307".to_string(),
        400 => "400".to_string(),
        401 => "401".to_string(),
        402 => "402".to_string(),
        403 => "403".to_string(),
        404 => "404".to_string(),
        405 => "405".to_string(),
        406 => "406".to_string(),
        407 => "407".to_string(),
        408 => "408".to_string(),
        409 => "409".to_string(),
        410 => "410".to_string(),
        411 => "411".to_string(),
        412 => "412".to_string(),
        413 => "413".to_string(),
        414 => "414".to_string(),
        415 => "415".to_string(),
        416 => "416".to_string(),
        417 => "417".to_string(),
        418 => "418".to_string(),
        500 => "500".to_string(),
        501 => "501".to_string(),
        502 => "502".to_string(),
        503 => "503".to_string(),
        504 => "504".to_string(),
        505 => "505".to_string(),
        428 => "428".to_string(),
        429 => "429".to_string(),
        431 => "431".to_string(),
        511 => "511".to_string(),
        _ if s >= 100 && s <= 599 => s.to_string(),
        _ => "unknown".to_string(),
    }
}
