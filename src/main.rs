use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use crate::collector::{Encoder, TextEncoder, Registry, IntCounterVec, Opts};
use slog::{Drain, Logger, o, info, warn, debug};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::form_urlencoded;

mod collector;
mod prometheus;

struct Handler {
    unfiltered_handler: Arc<dyn Fn(Request<Body>) -> Response<Body> + Send + Sync>,
    enabled_collectors: Vec<String>,
    exporter_metrics_registry: Registry,
    include_exporter_metrics: bool,
    max_requests: usize,
    logger: Logger,
}

impl Handler {
    fn new(include_exporter_metrics: bool, max_requests: usize, logger: Logger) -> Self {
        let exporter_metrics_registry = Registry::new();
        let unfiltered_handler = Arc::new(|_req| {
            Response::new(Body::from("Unfiltered metrics"))
        });

        let mut handler = Handler {
            unfiltered_handler,
            enabled_collectors: vec![],
            exporter_metrics_registry,
            include_exporter_metrics,
            max_requests,
            logger,
        };

        if handler.include_exporter_metrics {
            handler.exporter_metrics_registry
                .register(Box::new(prometheus::process_collector::ProcessCollector::for_self()))
                .unwrap();
            handler.exporter_metrics_registry
                .register(Box::new(prometheus::GoCollector::default()))
                .unwrap();
        }

        if let Err(err) = handler.inner_handler(&[]) {
            panic!("Couldn't create metrics handler: {}", err);
        }

        handler
    }

    async fn serve_http(self: Arc<Mutex<Handler>>, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let query_params: HashMap<_, _> = form_urlencoded::parse(req.uri().query().unwrap_or("").as_bytes()).into_owned().collect();
        let collects: Vec<String> = query_params.get("collect[]").map_or(vec![], |v| v.split(',').map(String::from).collect());
        debug!(self.lock().await.logger, "collect query:"; "collects" => ?collects);

        let excludes: Vec<String> = query_params.get("exclude[]").map_or(vec![], |v| v.split(',').map(String::from).collect());
        debug!(self.lock().await.logger, "exclude query:"; "excludes" => ?excludes);

        if collects.is_empty() && excludes.is_empty() {
            // No filters, use the prepared unfiltered handler.
            return Ok((self.lock().await.unfiltered_handler)(req));
        }

        if !collects.is_empty() && !excludes.is_empty() {
            debug!(self.lock().await.logger, "rejecting combined collect and exclude queries");
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Combined collect and exclude queries are not allowed."))
                .unwrap());
        }

        let filters: Vec<String>;
        if !excludes.is_empty() {
            // In exclude mode, filtered collectors = enabled - excluded.
            filters = self.lock().await.enabled_collectors.iter()
                .filter(|c| !excludes.contains(c))
                .cloned()
                .collect();
        } else {
            filters = collects.clone();
        }

        // To serve filtered metrics, we create a filtering handler on the fly.
        match self.lock().await.inner_handler(&filters).await {
            Ok(filtered_handler) => Ok(filtered_handler(req)),
            Err(err) => {
                warn!(self.lock().await.logger, "Couldn't create filtered metrics handler:"; "err" => ?err);
                Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from(format!("Couldn't create filtered metrics handler: {}", err)))
                    .unwrap())
            }
        }
    }

    async fn inner_handler(&self, filters: &[String]) -> Result<Arc<dyn Fn(Request<Body>) -> Response<Body> + Send + Sync>, Box<dyn std::error::Error>> {
        let nc = NodeCollector::new(&self.logger, filters).await?;
        let mut enabled_collectors = self.enabled_collectors.clone();

        // Only log the creation of an unfiltered handler, which should happen only once upon startup.
        if filters.is_empty() {
            info!(self.logger, "Enabled collectors");
            for n in nc.collectors.keys() {
                enabled_collectors.push(n.clone());
            }
            enabled_collectors.sort();
            for c in &enabled_collectors {
                info!(self.logger, "{}", c);
            }
        }

        let r = Registry::new();
        r.register(Box::new(prometheus::process_collector::ProcessCollector::for_self()))?;
        r.register(Box::new(prometheus::GoCollector::default()))?;
        r.register(Box::new(nc))?;

        let handler = if self.include_exporter_metrics {
            let gatherers = prometheus::Gatherers::new(vec![self.exporter_metrics_registry.clone(), r.clone()]);
            let handler = prometheus::prometheus::http::Handler::new(gatherers, self.max_requests);
            Arc::new(move |req: Request<Body>| {
                let encoder = TextEncoder::new();
                let metric_families = gatherers.gather();
                let mut buffer = Vec::new();
                encoder.encode(&metric_families, &mut buffer).unwrap();
                Response::new(Body::from(buffer))
            }) as Arc<dyn Fn(Request<Body>) -> Response<Body> + Send + Sync>
        } else {
            Arc::new(move |req: Request<Body>| {
                let encoder = TextEncoder::new();
                let metric_families = r.gather();
                let mut buffer = Vec::new();
                encoder.encode(&metric_families, &mut buffer).unwrap();
                Response::new(Body::from(buffer))
            }) as Arc<dyn Fn(Request<Body>) -> Response<Body> + Send + Sync>
        };

        Ok(handler)
    }
}

#[tokio::main]
async fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = Logger::root(drain, o!());

    let handler = Arc::new(Mutex::new(Handler::new(true, 40, logger.clone())));

    let make_svc = make_service_fn(move |_conn| {
        let handler = handler.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handler.clone().serve_http(req)
            }))
        }
    });

    let addr = ([127, 0, 0, 1], 9100).into();
    let server = Server::bind(&addr).serve(make_svc);

    info!(logger, "Starting node_exporter"; "version" => "1.0.0");
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}