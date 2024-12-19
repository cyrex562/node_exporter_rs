// Copyright 2020 The Prometheus Authors
// This code is partly borrowed from Caddy:
//    Copyright 2015 Matthew Holt and The Caddy Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bcrypt::{hash, verify, DEFAULT_COST};
use hex;
use hyper::{header, Body, Request, Response, StatusCode};
use log::error;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;

lazy_static::lazy_static! {
    static ref EXTRA_HTTP_HEADERS: HashMap<&'static str, Vec<&'static str>> = {
        let mut m = HashMap::new();
        m.insert("Strict-Transport-Security", vec![]);
        m.insert("X-Content-Type-Options", vec!["nosniff"]);
        m.insert("X-Frame-Options", vec!["deny", "sameorigin"]);
        m.insert("X-XSS-Protection", vec![]);
        m.insert("Content-Security-Policy", vec![]);
        m
    };
}

fn validate_users(config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = get_config(config_path)?;
    for p in config.users.values() {
        bcrypt::hash(p, DEFAULT_COST)?;
    }
    Ok(())
}

fn validate_header_config(headers: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    for (k, v) in headers {
        if let Some(values) = EXTRA_HTTP_HEADERS.get(k.as_str()) {
            if !values.is_empty() && !values.contains(&v.as_str()) {
                return Err(format!("invalid value for {}. Expected one of: {:?}, but got: {}", k, values, v).into());
            }
        } else {
            return Err(format!("HTTP header {} cannot be configured", k).into());
        }
    }
    Ok(())
}

struct WebHandler {
    tls_config_path: String,
    handler: Arc<dyn Fn(Request<Body>) -> Response<Body> + Send + Sync>,
    logger: slog::Logger,
    cache: Arc<Cache>,
    bcrypt_mtx: AsyncMutex<()>,
}

impl WebHandler {
    async fn serve_http(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let config = match get_config(&self.tls_config_path) {
            Ok(c) => c,
            Err(err) => {
                error!("Unable to parse configuration: {}", err);
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(StatusCode::INTERNAL_SERVER_ERROR.to_string()))
                    .unwrap());
            }
        };

        // Configure HTTP headers.
        let mut response = Response::new(Body::empty());
        for (k, v) in &config.http_config.header {
            response.headers_mut().insert(k, v.parse().unwrap());
        }

        if config.users.is_empty() {
            return Ok((self.handler)(req));
        }

        if let Some((user, pass)) = basic_auth(&req) {
            let hashed_password = config.users.get(&user).unwrap_or(&"$2y$10$QOauhQNbBCuQDKes6eFzPeMqBSjb7Mr5DUmpZ/VcEd00UAV/LDeSi".to_string());
            let cache_key = format!(
                "{}:{}:{}",
                hex::encode(user.as_bytes()),
                hex::encode(hashed_password.as_bytes()),
                hex::encode(pass.as_bytes())
            );

            let auth_ok = {
                let cache = self.cache.lock().unwrap();
                if let Some(&auth_ok) = cache.get(&cache_key) {
                    auth_ok
                } else {
                    drop(cache);
                    let _guard = self.bcrypt_mtx.lock().await;
                    let auth_ok = verify(pass, hashed_password).unwrap_or(false);
                    self.cache.lock().unwrap().insert(cache_key, auth_ok);
                    auth_ok
                }
            };

            if auth_ok {
                return Ok((self.handler)(req));
            }
        }

        response.headers_mut().insert(header::WWW_AUTHENTICATE, "Basic".parse().unwrap());
        *response.status_mut() = StatusCode::UNAUTHORIZED;
        Ok(response)
    }
}

fn basic_auth(req: &Request<Body>) -> Option<(String, String)> {
    req.headers().get(header::AUTHORIZATION).and_then(|header| {
        let header_value = header.to_str().ok()?;
        if !header_value.starts_with("Basic ") {
            return None;
        }
        let encoded = header_value.trim_start_matches("Basic ");
        let decoded = base64::decode(encoded).ok()?;
        let decoded_str = String::from_utf8(decoded).ok()?;
        let mut parts = decoded_str.splitn(2, ':');
        let user = parts.next()?.to_string();
        let pass = parts.next()?.to_string();
        Some((user, pass))
    })
}