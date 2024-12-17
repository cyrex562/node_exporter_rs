use hyper::header::ACCEPT;
use hyper::{Body, Request, Response, Server, StatusCode};
use prost::Message;
use prost_types::Any;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
enum Format {
    ProtoDelim,
    ProtoText,
    ProtoCompact,
    TextPlain,
    OpenMetrics,
}

impl Format {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "application/vnd.google.protobuf; proto=io.prometheus.client.MetricFamily; encoding=delimited" => {
                Some(Format::ProtoDelim)
            }
            "application/vnd.google.protobuf; proto=io.prometheus.client.MetricFamily; encoding=text" => {
                Some(Format::ProtoText)
            }
            "application/vnd.google.protobuf; proto=io.prometheus.client.MetricFamily; encoding=compact-text" => {
                Some(Format::ProtoCompact)
            }
            "text/plain" => Some(Format::TextPlain),
            "application/openmetrics-text" => Some(Format::OpenMetrics),
            _ => None,
        }
    }
}

struct Encoder {
    format: Format,
    writer: Box<dyn Write + Send>,
}

impl Encoder {
    fn new(format: Format, writer: Box<dyn Write + Send>) -> Self {
        Encoder { format, writer }
    }

    fn encode(&mut self, metric_family: &Any) -> Result<(), Box<dyn Error>> {
        match self.format {
            Format::ProtoDelim => {
                let mut buf = Vec::new();
                metric_family.encode_length_delimited(&mut buf)?;
                self.writer.write_all(&buf)?;
            }
            Format::ProtoText => {
                let text = format!("{:?}", metric_family);
                self.writer.write_all(text.as_bytes())?;
            }
            Format::ProtoCompact => {
                let text = format!("{:?}", metric_family);
                self.writer.write_all(text.as_bytes())?;
            }
            Format::TextPlain => {
                let text = format!("{:?}", metric_family);
                self.writer.write_all(text.as_bytes())?;
            }
            Format::OpenMetrics => {
                let text = format!("{:?}", metric_family);
                self.writer.write_all(text.as_bytes())?;
                self.writer.write_all(b"# EOF\n")?;
            }
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), Box<dyn Error>> {
        if let Format::OpenMetrics = self.format {
            self.writer.write_all(b"# EOF\n")?;
        }
        Ok(())
    }
}

fn negotiate(headers: &hyper::HeaderMap) -> Format {
    if let Some(accept_header) = headers.get(ACCEPT) {
        let accept_str = accept_header.to_str().unwrap_or("");
        if let Some(format) = Format::from_str(accept_str) {
            return format;
        }
    }
    Format::TextPlain
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let format = negotiate(req.headers());
    let mut encoder = Encoder::new(format, Box::new(Vec::new()));

    let metric_family = Any {
        type_url: "type.googleapis.com/io.prometheus.client.MetricFamily".to_string(),
        value: vec![],
    };

    if let Err(e) = encoder.encode(&metric_family) {
        eprintln!("Failed to encode metric family: {}", e);
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Internal Server Error"))
            .unwrap());
    }

    if let Err(e) = encoder.close() {
        eprintln!("Failed to close encoder: {}", e);
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Internal Server Error"))
            .unwrap());
    }

    let body = encoder.writer.into_inner().unwrap();
    Ok(Response::new(Body::from(body)))
}