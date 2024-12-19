use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tokio::fs::read;

lazy_static::lazy_static! {
    static ref MIME_TYPES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert(".cjs", "application/javascript");
        m.insert(".css", "text/css");
        m.insert(".eot", "font/eot");
        m.insert(".gif", "image/gif");
        m.insert(".ico", "image/x-icon");
        m.insert(".jpg", "image/jpeg");
        m.insert(".js", "application/javascript");
        m.insert(".json", "application/json");
        m.insert(".less", "text/plain");
        m.insert(".map", "application/json");
        m.insert(".otf", "font/otf");
        m.insert(".png", "image/png");
        m.insert(".svg", "image/svg+xml");
        m.insert(".ttf", "font/ttf");
        m.insert(".txt", "text/plain");
        m.insert(".woff", "font/woff");
        m.insert(".woff2", "font/woff2");
        m
    };
}

async fn static_file_server(req: Request<Body>, root: &str) -> Result<Response<Body>, hyper::Error> {
    let path = format!("{}{}", root, req.uri().path());
    let file_ext = Path::new(&path).extension().and_then(|ext| ext.to_str()).unwrap_or("");

    let mut response = Response::new(Body::empty());

    if let Some(mime_type) = MIME_TYPES.get(file_ext) {
        response.headers_mut().insert("Content-Type", mime_type.parse().unwrap());
    }

    match read(&path).await {
        Ok(contents) => {
            *response.body_mut() = Body::from(contents);
        }
        Err(_) => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    }

    Ok(response)
}