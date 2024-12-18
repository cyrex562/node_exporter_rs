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
use std::io::{self, BufRead, Read};
use std::str::FromStr;

use hyper::header::HeaderMap;
use mime::Mime;
use prost::Message;
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::family::MetricFamily;
use prometheus_client::metrics::MetricType;
use prometheus_client::proto::MetricFamily as ProtoMetricFamily;
use prometheus_client::proto::MetricType as ProtoMetricType;
use prometheus_client::registry::Registry;
use prometheus_client::timestamp::Timestamp;

pub trait Decoder {
    fn decode(&mut self, metric_family: &mut ProtoMetricFamily) -> io::Result<()>;
}

pub struct DecodeOptions {
    pub timestamp: Option<Timestamp>,
}

pub fn response_format(headers: &HeaderMap) -> Format {
    if let Some(content_type) = headers.get(hyper::header::CONTENT_TYPE) {
        if let Ok(content_type) = content_type.to_str() {
            if let Ok(mime) = Mime::from_str(content_type) {
                return match (mime.type_(), mime.subtype()) {
                    (mime::APPLICATION, mime::PROTOBUF) => Format::Proto,
                    (mime::TEXT, mime::PLAIN) => Format::Text,
                    _ => Format::Unknown,
                };
            }
        }
    }
    Format::Unknown
}

pub fn new_decoder<R: Read>(reader: R, format: Format) -> Box<dyn Decoder> {
    match format {
        Format::Proto => Box::new(ProtoDecoder::new(reader)),
        Format::Text => Box::new(TextDecoder::new(reader)),
        Format::Unknown => Box::new(TextDecoder::new(reader)),
    }
}

pub struct ProtoDecoder<R: Read> {
    reader: R,
}

impl<R: Read> ProtoDecoder<R> {
    pub fn new(reader: R) -> Self {
        ProtoDecoder { reader }
    }
}

impl<R: Read> Decoder for ProtoDecoder<R> {
    fn decode(&mut self, metric_family: &mut ProtoMetricFamily) -> io::Result<()> {
        let mut buf = Vec::new();
        self.reader.read_to_end(&mut buf)?;
        metric_family.merge(buf.as_slice()).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

pub struct TextDecoder<R: Read> {
    reader: R,
    fams: HashMap<String, ProtoMetricFamily>,
    err: Option<io::Error>,
}

impl<R: Read> TextDecoder<R> {
    pub fn new(reader: R) -> Self {
        TextDecoder {
            reader,
            fams: HashMap::new(),
            err: None,
        }
    }
}

impl<R: Read> Decoder for TextDecoder<R> {
    fn decode(&mut self, metric_family: &mut ProtoMetricFamily) -> io::Result<()> {
        if self.err.is_none() {
            let mut buf_reader = BufRead::new(&mut self.reader);
            let mut parser = TextParser::new();
            self.fams = parser.text_to_metric_families(&mut buf_reader)?;
            self.err = Some(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }

        if let Some((_, fam)) = self.fams.iter().next() {
            *metric_family = fam.clone();
            self.fams.remove(metric_family.get_name());
            Ok(())
        } else {
            self.err.clone().unwrap()
        }
    }
}

pub struct SampleDecoder<D: Decoder> {
    decoder: D,
    options: DecodeOptions,
    metric_family: ProtoMetricFamily,
}

impl<D: Decoder> SampleDecoder<D> {
    pub fn new(decoder: D, options: DecodeOptions) -> Self {
        SampleDecoder {
            decoder,
            options,
            metric_family: ProtoMetricFamily::default(),
        }
    }

    pub fn decode(&mut self, samples: &mut Vec<Sample>) -> io::Result<()> {
        self.decoder.decode(&mut self.metric_family)?;
        *samples = extract_samples(&self.metric_family, &self.options)?;
        Ok(())
    }
}

pub fn extract_samples(
    metric_family: &ProtoMetricFamily,
    options: &DecodeOptions,
) -> io::Result<Vec<Sample>> {
    match metric_family.get_field_type() {
        ProtoMetricType::COUNTER => extract_counter(metric_family, options),
        ProtoMetricType::GAUGE => extract_gauge(metric_family, options),
        ProtoMetricType::SUMMARY => extract_summary(metric_family, options),
        ProtoMetricType::UNTYPED => extract_untyped(metric_family, options),
        ProtoMetricType::HISTOGRAM => extract_histogram(metric_family, options),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unknown metric family type",
        )),
    }
}

fn extract_counter(
    metric_family: &ProtoMetricFamily,
    options: &DecodeOptions,
) -> io::Result<Vec<Sample>> {
    let mut samples = Vec::new();
    for metric in metric_family.get_metric() {
        if let Some(counter) = metric.get_counter() {
            let mut labels = HashMap::new();
            for label in metric.get_label() {
                labels.insert(label.get_name().to_string(), label.get_value().to_string());
            }
            labels.insert("__name__".to_string(), metric_family.get_name().to_string());

            let sample = Sample {
                metric: labels,
                value: counter.get_value(),
                timestamp: metric.get_timestamp_ms().map_or(options.timestamp, |ts| {
                    Some(Timestamp::from_millis(ts as i64))
                }),
            };
            samples.push(sample);
        }
    }
    Ok(samples)
}

fn extract_gauge(
    metric_family: &ProtoMetricFamily,
    options: &DecodeOptions,
) -> io::Result<Vec<Sample>> {
    let mut samples = Vec::new();
    for metric in metric_family.get_metric() {
        if let Some(gauge) = metric.get_gauge() {
            let mut labels = HashMap::new();
            for label in metric.get_label() {
                labels.insert(label.get_name().to_string(), label.get_value().to_string());
            }
            labels.insert("__name__".to_string(), metric_family.get_name().to_string());

            let sample = Sample {
                metric: labels,
                value: gauge.get_value(),
                timestamp: metric.get_timestamp_ms().map_or(options.timestamp, |ts| {
                    Some(Timestamp::from_millis(ts as i64))
                }),
            };
            samples.push(sample);
        }
    }
    Ok(samples)
}

fn extract_untyped(
    metric_family: &ProtoMetricFamily,
    options: &DecodeOptions,
) -> io::Result<Vec<Sample>> {
    let mut samples = Vec::new();
    for metric in metric_family.get_metric() {
        if let Some(untyped) = metric.get_untyped() {
            let mut labels = HashMap::new();
            for label in metric.get_label() {
                labels.insert(label.get_name().to_string(), label.get_value().to_string());
            }
            labels.insert("__name__".to_string(), metric_family.get_name().to_string());

            let sample = Sample {
                metric: labels,
                value: untyped.get_value(),
                timestamp: metric.get_timestamp_ms().map_or(options.timestamp, |ts| {
                    Some(Timestamp::from_millis(ts as i64))
                }),
            };
            samples.push(sample);
        }
    }
    Ok(samples)
}

fn extract_summary(
    metric_family: &ProtoMetricFamily,
    options: &DecodeOptions,
) -> io::Result<Vec<Sample>> {
    let mut samples = Vec::new();
    for metric in metric_family.get_metric() {
        if let Some(summary) = metric.get_summary() {
            let timestamp = metric.get_timestamp_ms().map_or(options.timestamp, |ts| {
                Some(Timestamp::from_millis(ts as i64))
            });

            for quantile in summary.get_quantile() {
                let mut labels = HashMap::new();
                for label in metric.get_label() {
                    labels.insert(label.get_name().to_string(), label.get_value().to_string());
                }
                labels.insert("__name__".to_string(), metric_family.get_name().to_string());
                labels.insert("quantile".to_string(), quantile.get_quantile().to_string());

                let sample = Sample {
                    metric: labels,
                    value: quantile.get_value(),
                    timestamp,
                };
                samples.push(sample);
            }

            let mut labels = HashMap::new();
            for label in metric.get_label() {
                labels.insert(label.get_name().to_string(), label.get_value().to_string());
            }
            labels.insert("__name__".to_string(), format!("{}_sum", metric_family.get_name()));

            let sample = Sample {
                metric: labels,
                value: summary.get_sample_sum(),
                timestamp,
            };
            samples.push(sample);

            let mut labels = HashMap::new();
            for label in metric.get_label() {
                labels.insert(label.get_name().to_string(), label.get_value().to_string());
            }
            labels.insert("__name__".to_string(), format!("{}_count", metric_family.get_name()));

            let sample = Sample {
                metric: labels,
                value: summary.get_sample_count() as f64,
                timestamp,
            };
            samples.push(sample);
        }
    }
    Ok(samples)
}

fn extract_histogram(
    metric_family: &ProtoMetricFamily,
    options: &DecodeOptions,
) -> io::Result<Vec<Sample>> {
    let mut samples = Vec::new();
    for metric in metric_family.get_metric() {
        if let Some(histogram) = metric.get_histogram() {
            let timestamp = metric.get_timestamp_ms().map_or(options.timestamp, |ts| {
                Some(Timestamp::from_millis(ts as i64))
            });

            let mut inf_seen = false;

            for bucket in histogram.get_bucket() {
                let mut labels = HashMap::new();
                for label in metric.get_label() {
                    labels.insert(label.get_name().to_string(), label.get_value().to_string());
                }
                labels.insert("__name__".to_string(), format!("{}_bucket", metric_family.get_name()));
                labels.insert("le".to_string(), bucket.get_upper_bound().to_string());

                if bucket.get_upper_bound().is_infinite() {
                    inf_seen = true;
                }

                let sample = Sample {
                    metric: labels,
                    value: bucket.get_cumulative_count() as f64,
                    timestamp,
                };
                samples.push(sample);
            }

            let mut labels = HashMap::new();
            for label in metric.get_label() {
                labels.insert(label.get_name().to_string(), label.get_value().to_string());
            }
            labels.insert("__name__".to_string(), format!("{}_sum", metric_family.get_name()));

            let sample = Sample {
                metric: labels,
                value: histogram.get_sample_sum(),
                timestamp,
            };
            samples.push(sample);

            let mut labels = HashMap::new();
            for label in metric.get_label() {
                labels.insert(label.get_name().to_string(), label.get_value().to_string());
            }
            labels.insert("__name__".to_string(), format!("{}_count", metric_family.get_name()));

            let count_sample = Sample {
                metric: labels,
                value: histogram.get_sample_count() as f64,
                timestamp,
            };
            samples.push(count_sample);

            if !inf_seen {
                let mut labels = HashMap::new();
                for label in metric.get_label() {
                    labels.insert(label.get_name().to_string(), label.get_value().to_string());
                }
                labels.insert("__name__".to_string(), format!("{}_bucket", metric_family.get_name()));
                labels.insert("le".to_string(), "+Inf".to_string());

                let sample = Sample {
                    metric: labels,
                    value: count_sample.value,
                    timestamp,
                };
                samples.push(sample);
            }
        }
    }
    Ok(samples)
}

pub struct Sample {
    pub metric: HashMap<String, String>,
    pub value: f64,
    pub timestamp: Option<Timestamp>,
}

pub enum Format {
    Proto,
    Text,
    Unknown,
}

impl Format {
    pub fn format_type(&self) -> FormatType {
        match self {
            Format::Proto => FormatType::Proto,
            Format::Text => FormatType::Text,
            Format::Unknown => FormatType::Unknown,
        }
    }
}

pub enum FormatType {
    Proto,
    Text,
    Unknown,
}

pub struct TextParser;

impl TextParser {
    pub fn new() -> Self {
        TextParser
    }

    pub fn text_to_metric_families<R: BufRead>(
        &mut self,
        reader: &mut R,
    ) -> io::Result<HashMap<String, ProtoMetricFamily>> {
        let mut families = HashMap::new();
        let mut buffer = String::new();
        while reader.read_line(&mut buffer)? > 0 {
            let family = ProtoMetricFamily::decode(buffer.as_bytes()).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Failed to decode: {}", e))
            })?;
            families.insert(family.get_name().to_string(), family);
            buffer.clear();
        }
        Ok(families)
    }
}