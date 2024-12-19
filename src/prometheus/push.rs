// Copyright 2015 The Prometheus Authors
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

//! Package push provides functions to push metrics to a Pushgateway. It uses a
//! builder approach. Create a Pusher with `new` and then add the various options
//! by using its methods, finally calling `add` or `push`, like this:
//!
//! ```rust
//! // Easy case:
//! push::Pusher::new("http://example.org/metrics", "my_job").gatherer(my_registry).push();
//!
//! // Complex case:
//! push::Pusher::new("http://example.org/metrics", "my_job")
//!     .collector(my_collector1)
//!     .collector(my_collector2)
//!     .grouping("zone", "xy")
//!     .client(&my_http_client)
//!     .basic_auth("top", "secret")
//!     .add();
//! ```
//!
//! See the examples section for more detailed examples.
//!
//! See the documentation of the Pushgateway to understand the meaning of
//! the grouping key and the differences between `push` and `add`:
//! https://github.com/prometheus/pushgateway

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::{self, Read};
use std::sync::Arc;
use std::time::Duration;
use base64::encode_config;
use hyper::{Body, Client, Method, Request, Response, StatusCode};
use hyper::client::HttpConnector;
use hyper::header::{HeaderValue, CONTENT_TYPE};
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::Uri;
use prometheus::{self, Encoder, TextEncoder, Registry, Gatherer, Collector};
use tokio::time::timeout;

const CONTENT_TYPE_HEADER: &str = "Content-Type";
const BASE64_SUFFIX: &str = "@base64";

#[derive(Debug)]
pub struct Pusher {
    error: Option<Box<dyn Error>>,
    url: String,
    job: String,
    grouping: HashMap<String, String>,
    gatherers: Vec<Arc<dyn Gatherer>>,
    registerer: Arc<Registry>,
    client: Client<HttpConnector>,
    header: Option<hyper::HeaderMap>,
    use_basic_auth: bool,
    username: String,
    password: String,
    expfmt: String,
}

impl Pusher {
    pub fn new(url: &str, job: &str) -> Self {
        let reg = Arc::new(Registry::new());
        let url = if !url.contains("://") {
            format!("http://{}", url)
        } else {
            url.to_string()
        };
        let url = url.trim_end_matches('/').to_string();

        Pusher {
            error: if job.is_empty() { Some(Box::new(fmt::Error)) } else { None },
            url,
            job: job.to_string(),
            grouping: HashMap::new(),
            gatherers: vec![reg.clone()],
            registerer: reg,
            client: Client::new(),
            header: None,
            use_basic_auth: false,
            username: String::new(),
            password: String::new(),
            expfmt: "application/vnd.google.protobuf; proto=io.prometheus.client.MetricFamily; encoding=delimited".to_string(),
        }
    }

    pub fn push(&self) -> Result<(), Box<dyn Error>> {
        self.push_with_method(Method::PUT)
    }

    pub fn push_with_context(&self, ctx: &tokio::runtime::Handle) -> Result<(), Box<dyn Error>> {
        self.push_with_method_and_context(Method::PUT, ctx)
    }

    pub fn add(&self) -> Result<(), Box<dyn Error>> {
        self.push_with_method(Method::POST)
    }

    pub fn add_with_context(&self, ctx: &tokio::runtime::Handle) -> Result<(), Box<dyn Error>> {
        self.push_with_method_and_context(Method::POST, ctx)
    }

    pub fn gatherer(mut self, g: Arc<dyn Gatherer>) -> Self {
        self.gatherers.push(g);
        self
    }

    pub fn collector(mut self, c: Arc<dyn Collector>) -> Self {
        if self.error.is_none() {
            self.error = self.registerer.register(c).err().map(|e| Box::new(e) as Box<dyn Error>);
        }
        self
    }

    pub fn error(&self) -> Option<&Box<dyn Error>> {
        self.error.as_ref()
    }

    pub fn grouping(mut self, name: &str, value: &str) -> Self {
        if self.error.is_none() {
            if !prometheus::Label::is_valid_name(name) {
                self.error = Some(Box::new(fmt::Error));
                return self;
            }
            self.grouping.insert(name.to_string(), value.to_string());
        }
        self
    }

    pub fn client(mut self, c: Client<HttpConnector>) -> Self {
        self.client = c;
        self
    }

    pub fn header(mut self, header: hyper::HeaderMap) -> Self {
        self.header = Some(header);
        self
    }

    pub fn basic_auth(mut self, username: &str, password: &str) -> Self {
        self.use_basic_auth = true;
        self.username = username.to_string();
        self.password = password.to_string();
        self
    }

    pub fn format(mut self, format: &str) -> Self {
        self.expfmt = format.to_string();
        self
    }

    pub fn delete(&self) -> Result<(), Box<dyn Error>> {
        if let Some(ref err) = self.error {
            return Err(err.clone());
        }
        let req = Request::builder()
            .method(Method::DELETE)
            .uri(self.full_url())
            .body(Body::empty())?;
        let res = self.client.request(req).await?;
        if res.status() != StatusCode::ACCEPTED {
            let body = hyper::body::to_bytes(res.into_body()).await?;
            return Err(Box::new(io::Error::new(io::ErrorKind::Other, format!(
                "unexpected status code {} while deleting {}: {}",
                res.status(),
                self.full_url(),
                String::from_utf8_lossy(&body)
            ))));
        }
        Ok(())
    }

    fn push_with_method(&self, method: Method) -> Result<(), Box<dyn Error>> {
        self.push_with_method_and_context(method, &tokio::runtime::Handle::current())
    }

    fn push_with_method_and_context(&self, method: Method, ctx: &tokio::runtime::Handle) -> Result<(), Box<dyn Error>> {
        if let Some(ref err) = self.error {
            return Err(err.clone());
        }
        let mfs = self.gatherers.iter().flat_map(|g| g.gather()).collect::<Vec<_>>();
        let mut buf = Vec::new();
        let encoder = TextEncoder::new();
        for mf in mfs {
            for m in mf.get_metric() {
                for l in m.get_label() {
                    if l.get_name() == "job" {
                        return Err(Box::new(fmt::Error));
                    }
                    if self.grouping.contains_key(l.get_name()) {
                        return Err(Box::new(fmt::Error));
                    }
                }
            }
            encoder.encode(&[mf], &mut buf)?;
        }
        let req = Request::builder()
            .method(method)
            .uri(self.full_url())
            .header(CONTENT_TYPE, self.expfmt.clone())
            .body(Body::from(buf))?;
        let res = self.client.request(req).await?;
        if res.status() != StatusCode::OK && res.status() != StatusCode::ACCEPTED {
            let body = hyper::body::to_bytes(res.into_body()).await?;
            return Err(Box::new(io::Error::new(io::ErrorKind::Other, format!(
                "unexpected status code {} while pushing to {}: {}",
                res.status(),
                self.full_url(),
                String::from_utf8_lossy(&body)
            ))));
        }
        Ok(())
    }

    fn full_url(&self) -> String {
        let mut url_components = vec![];
        if let (encoded_job, true) = encode_component(&self.job) {
            url_components.push(format!("job{}", BASE64_SUFFIX));
            url_components.push(encoded_job);
        } else {
            url_components.push("job".to_string());
            url_components.push(encode_component(&self.job).0);
        }
        for (ln, lv) in &self.grouping {
            if let (encoded_lv, true) = encode_component(lv) {
                url_components.push(format!("{}{}", ln, BASE64_SUFFIX));
                url_components.push(encoded_lv);
            } else {
                url_components.push(ln.clone());
                url_components.push(encode_component(lv).0);
            }
        }
        format!("{}/metrics/{}", self.url, url_components.join("/"))
    }
}

fn encode_component(s: &str) -> (String, bool) {
    if s.is_empty() {
        return ("=".to_string(), true);
    }
    if s.contains('/') {
        return (encode_config(s, base64::URL_SAFE), true);
    }
    (urlencoding::encode(s).to_string(), false)
}