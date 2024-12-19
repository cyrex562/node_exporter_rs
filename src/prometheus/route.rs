use std::sync::Arc;
use std::collections::HashMap;
use hyper::{Body, Request, Response, Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::server::Server;
use hyper::header::HeaderValue;
use tokio::sync::Mutex;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};

type Handler = Arc<dyn Fn(Request<Body>) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>> + Send + Sync>;

struct Router {
    routes: Arc<Mutex<HashMap<(Method, String), Handler>>>,
    prefix: String,
    instrh: Option<Arc<dyn Fn(&str, Handler) -> Handler + Send + Sync>>,
}

impl Router {
    fn new() -> Self {
        Router {
            routes: Arc::new(Mutex::new(HashMap::new())),
            prefix: String::new(),
            instrh: None,
        }
    }

    fn with_instrumentation(mut self, instrh: Arc<dyn Fn(&str, Handler) -> Handler + Send + Sync>) -> Self {
        self.instrh = Some(instrh);
        self
    }

    fn with_prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    fn handle(&self, handler_name: &str, h: Handler) -> Handler {
        if let Some(instrh) = &self.instrh {
            instrh(handler_name, h)
        } else {
            h
        }
    }

    async fn add_route(&self, method: Method, path: &str, h: Handler) {
        let mut routes = self.routes.lock().await;
        routes.insert((method, self.prefix.clone() + path), self.handle(path, h));
    }

    async fn get(&self, path: &str, h: Handler) {
        self.add_route(Method::GET, path, h).await;
    }

    async fn options(&self, path: &str, h: Handler) {
        self.add_route(Method::OPTIONS, path, h).await;
    }

    async fn delete(&self, path: &str, h: Handler) {
        self.add_route(Method::DELETE, path, h).await;
    }

    async fn put(&self, path: &str, h: Handler) {
        self.add_route(Method::PUT, path, h).await;
    }

    async fn post(&self, path: &str, h: Handler) {
        self.add_route(Method::POST, path, h).await;
    }

    async fn head(&self, path: &str, h: Handler) {
        self.add_route(Method::HEAD, path, h).await;
    }

    async fn redirect(&self, req: Request<Body>, path: &str, code: StatusCode) -> Result<Response<Body>, Infallible> {
        let uri = req.uri().clone();
        let new_uri = format!("{}{}", self.prefix, path);
        let response = Response::builder()
            .status(code)
            .header("Location", HeaderValue::from_str(&new_uri).unwrap())
            .body(Body::empty())
            .unwrap();
        Ok(response)
    }

    async fn serve_http(self: Arc<Self>, req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let routes = self.routes.lock().await;
        if let Some(handler) = routes.get(&(req.method().clone(), req.uri().path().to_string())) {
            handler(req).await
        } else {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap())
        }
    }

    fn file_serve(dir: &str) -> Handler {
        let dir = dir.to_string();
        Arc::new(move |req: Request<Body>| {
            let dir = dir.clone();
            Box::pin(async move {
                let path = req.uri().path().to_string();
                let file_path = format!("{}/{}", dir, path);
                match tokio::fs::read(file_path).await {
                    Ok(contents) => Ok(Response::builder()
                        .status(StatusCode::OK)
                        .body(Body::from(contents))
                        .unwrap()),
                    Err(_) => Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::from("File Not Found"))
                        .unwrap()),
                }
            })
        })
    }
}