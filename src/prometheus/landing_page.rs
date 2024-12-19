// Copyright 2023 The Prometheus Authors
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

use std::collections::HashMap;
use std::sync::Arc;
use hyper::{Body, Request, Response, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use tokio::sync::Mutex;
use askama::Template;

#[derive(Template)]
#[template(path = "landing_page.html")]
struct LandingPageTemplate<'a> {
    header_color: &'a str,
    css: &'a str,
    name: &'a str,
    description: &'a str,
    form: &'a LandingForm,
    links: &'a [LandingLinks],
    extra_html: &'a str,
    extra_css: &'a str,
    version: &'a str,
}

struct LandingConfig {
    header_color: String,
    css: String,
    name: String,
    description: String,
    form: LandingForm,
    links: Vec<LandingLinks>,
    extra_html: String,
    extra_css: String,
    version: String,
}

struct LandingForm {
    action: String,
    inputs: Vec<LandingFormInput>,
    width: f64,
}

struct LandingFormInput {
    label: String,
    input_type: String,
    name: String,
    placeholder: String,
    value: String,
}

struct LandingLinks {
    address: String,
    text: String,
    description: String,
}

struct LandingPageHandler {
    landing_page: Vec<u8>,
}

impl LandingPageHandler {
    async fn new(config: LandingConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let mut max_label_length = 0;
        for input in &config.form.inputs {
            if input.label.len() > max_label_length {
                max_label_length = input.label.len();
            }
        }
        let mut config = config;
        config.form.width = (max_label_length as f64 + 1.0) / 2.0;

        if config.css.is_empty() {
            if config.header_color.is_empty() {
                config.header_color = "#e6522c".to_string();
            }
            let css_template = LandingPageTemplate {
                header_color: &config.header_color,
                css: "",
                name: &config.name,
                description: &config.description,
                form: &config.form,
                links: &config.links,
                extra_html: &config.extra_html,
                extra_css: &config.extra_css,
                version: &config.version,
            };
            config.css = css_template.render()?;
        }

        let html_template = LandingPageTemplate {
            header_color: &config.header_color,
            css: &config.css,
            name: &config.name,
            description: &config.description,
            form: &config.form,
            links: &config.links,
            extra_html: &config.extra_html,
            extra_css: &config.extra_css,
            version: &config.version,
        };

        let landing_page = html_template.render()?.into_bytes();

        Ok(LandingPageHandler { landing_page })
    }

    async fn serve_http(self: Arc<Self>, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        if req.uri().path() != "/" {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap());
        }

        Ok(Response::builder()
            .header("Content-Type", "text/html; charset=UTF-8")
            .body(Body::from(self.landing_page.clone()))
            .unwrap())
    }
}