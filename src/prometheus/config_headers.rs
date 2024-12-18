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

// This module no longer handles safe YAML parsing. To ensure correct YAML unmarshalling,
// use `serde_yaml::from_str`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io};
use hyper::{Body, Request, Response};
use hyper::client::HttpConnector;
use hyper::header::{HeaderMap, HeaderName, HeaderValue};
use hyper::client::Client;
use hyper::rt::Future;
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    // Reserved headers that change the connection, are set by Prometheus, or can be changed otherwise.
    static ref RESERVED_HEADERS: Vec<&'static str> = vec![
        "Authorization",
        "Host",
        "Content-Encoding",
        "Content-Length",
        "Content-Type",
        "User-Agent",
        "Connection",
        "Keep-Alive",
        "Proxy-Authenticate",
        "Proxy-Authorization",
        "Www-Authenticate",
        "Accept-Encoding",
        "X-Prometheus-Remote-Write-Version",
        "X-Prometheus-Remote-Read-Version",
        "X-Prometheus-Scrape-Timeout-Seconds",
        // Added by SigV4.
        "X-Amz-Date",
        "X-Amz-Security-Token",
        "X-Amz-Content-Sha256",
    ];
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Headers {
    #[serde(flatten)]
    pub headers: HashMap<String, Header>,
}

impl Headers {
    pub fn set_directory(&mut self, dir: &Path) {
        for header in self.headers.values_mut() {
            header.set_directory(dir);
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        for name in self.headers.keys() {
            let canonical_name = name.to_ascii_lowercase();
            if RESERVED_HEADERS.contains(&canonical_name.as_str()) {
                return Err(format!("setting header {} is not allowed", canonical_name));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Header {
    #[serde(default)]
    pub values: Vec<String>,
    #[serde(default)]
    pub secrets: Vec<Secret>,
    #[serde(default)]
    pub files: Vec<String>,
}

impl Header {
    pub fn set_directory(&mut self, dir: &Path) {
        for file in &mut self.files {
            *file = join_dir(dir, Path::new(file)).to_string_lossy().into_owned();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret(String);

impl From<String> for Secret {
    fn from(s: String) -> Self {
        Secret(s)
    }
}

impl Secret {
    pub fn new(value: String) -> Self {
        Secret(value)
    }
}

pub struct HeadersLayer {
    config: Headers,
}

impl HeadersLayer {
    pub fn new(config: Headers) -> Self {
        HeadersLayer { config }
    }

    pub fn intercept_request(&self, mut req: Request<Body>) -> Result<Request<Body>, io::Error> {
        let mut headers = req.headers_mut();
        for (name, header) in &self.config.headers {
            let header_name = HeaderName::from_bytes(name.as_bytes()).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid header name: {}", name),
                )
            })?;
            for value in &header.values {
                headers.append(&header_name, HeaderValue::from_str(value)?);
            }
            for secret in &header.secrets {
                headers.append(&header_name, HeaderValue::from_str(&secret.0)?);
            }
            for file_path in &header.files {
                let content = fs::read_to_string(file_path)?;
                headers.append(
                    &header_name,
                    HeaderValue::from_str(content.trim())?,
                );
            }
        }
        Ok(req)
    }
}

fn join_dir(dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        dir.join(path)
    }
}

// Example usage with Hyper client:
pub fn make_request(url: &str, headers_config: Headers) -> impl Future<Item = Response<Body>, Error = hyper::Error> {
    let client = Client::new();
    let uri = url.parse().unwrap();
    let req = Request::get(uri).body(Body::empty()).unwrap();

    let headers_layer = HeadersLayer::new(headers_config);
    let req = headers_layer.intercept_request(req).unwrap();

    client.request(req)
}