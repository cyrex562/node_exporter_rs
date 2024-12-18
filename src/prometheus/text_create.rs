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

use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::family::MetricFamily;
use prometheus_client::metrics::MetricType;
use prometheus_client::proto::MetricFamily as ProtoMetricFamily;
use prometheus_client::proto::MetricType as ProtoMetricType;
use prometheus_client::registry::Registry;
use prometheus_client::timestamp::Timestamp;

type EnhancedWriter = Box<dyn Write + Send>;

const INITIAL_NUM_BUF_SIZE: usize = 24;

lazy_static::lazy_static! {
    static ref BUF_POOL: Arc<Mutex<Vec<EnhancedWriter>>> = Arc::new(Mutex::new(Vec::new()));
    static ref NUM_BUF_POOL: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
}

pub fn metric_family_to_text(out: &mut dyn Write, in_: &ProtoMetricFamily) -> io::Result<usize> {
    if in_.metric.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "MetricFamily has no metrics"));
    }
    let name = in_.get_name();
    if name.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "MetricFamily has no name"));
    }

    let mut w: EnhancedWriter = if let Some(writer) = BUF_POOL.lock().unwrap().pop() {
        writer
    } else {
        Box::new(io::BufWriter::new(out))
    };

    let mut written = 0;

    if let Some(help) = &in_.help {
        written += writeln!(w, "# HELP {} {}", name, escape_string(help, false))?;
    }

    written += writeln!(w, "# TYPE {} {}", name, metric_type_to_str(in_.get_field_type()))?;

    for metric in &in_.metric {
        match in_.get_field_type() {
            ProtoMetricType::COUNTER => {
                if let Some(counter) = &metric.counter {
                    written += write_sample(&mut w, name, "", metric, "", 0.0, counter.value)?;
                } else {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("expected counter in metric {}", name)));
                }
            }
            ProtoMetricType::GAUGE => {
                if let Some(gauge) = &metric.gauge {
                    written += write_sample(&mut w, name, "", metric, "", 0.0, gauge.value)?;
                } else {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("expected gauge in metric {}", name)));
                }
            }
            ProtoMetricType::UNTYPED => {
                if let Some(untyped) = &metric.untyped {
                    written += write_sample(&mut w, name, "", metric, "", 0.0, untyped.value)?;
                } else {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("expected untyped in metric {}", name)));
                }
            }
            ProtoMetricType::SUMMARY => {
                if let Some(summary) = &metric.summary {
                    for quantile in &summary.quantile {
                        written += write_sample(&mut w, name, "", metric, "quantile", quantile.quantile, quantile.value)?;
                    }
                    written += write_sample(&mut w, name, "_sum", metric, "", 0.0, summary.sample_sum)?;
                    written += write_sample(&mut w, name, "_count", metric, "", 0.0, summary.sample_count as f64)?;
                } else {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("expected summary in metric {}", name)));
                }
            }
            ProtoMetricType::HISTOGRAM => {
                if let Some(histogram) = &metric.histogram {
                    let mut inf_seen = false;
                    for bucket in &histogram.bucket {
                        written += write_sample(&mut w, name, "_bucket", metric, "le", bucket.upper_bound, bucket.cumulative_count as f64)?;
                        if bucket.upper_bound.is_infinite() {
                            inf_seen = true;
                        }
                    }
                    if !inf_seen {
                        written += write_sample(&mut w, name, "_bucket", metric, "le", f64::INFINITY, histogram.sample_count as f64)?;
                    }
                    written += write_sample(&mut w, name, "_sum", metric, "", 0.0, histogram.sample_sum)?;
                    written += write_sample(&mut w, name, "_count", metric, "", 0.0, histogram.sample_count as f64)?;
                } else {
                    return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("expected histogram in metric {}", name)));
                }
            }
            _ => {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("unexpected type in metric {}", name)));
            }
        }
    }

    BUF_POOL.lock().unwrap().push(w);
    Ok(written)
}

fn write_sample(
    w: &mut EnhancedWriter,
    name: &str,
    suffix: &str,
    metric: &ProtoMetricFamily,
    additional_label_name: &str,
    additional_label_value: f64,
    value: f64,
) -> io::Result<usize> {
    let mut written = 0;
    written += write_name_and_label_pairs(w, name, suffix, &metric.label, additional_label_name, additional_label_value)?;
    written += write!(w, " ")?;
    written += write_float(w, value)?;
    if let Some(timestamp) = &metric.timestamp_ms {
        written += write!(w, " ")?;
        written += write_int(w, *timestamp)?;
    }
    written += write!(w, "\n")?;
    Ok(written)
}

fn write_name_and_label_pairs(
    w: &mut EnhancedWriter,
    name: &str,
    suffix: &str,
    labels: &[ProtoMetricFamily],
    additional_label_name: &str,
    additional_label_value: f64,
) -> io::Result<usize> {
    let mut written = 0;
    let mut separator = '{';

    if !name.is_empty() {
        if !is_valid_legacy_metric_name(name) {
            written += write!(w, "{}\"{}\"", separator, name)?;
            separator = ',';
        } else {
            written += write!(w, "{}", name)?;
        }
    }

    for label in labels {
        written += write!(w, "{}{}=\"{}\"", separator, label.get_name(), escape_string(label.get_value(), true))?;
        separator = ',';
    }

    if !additional_label_name.is_empty() {
        written += write!(w, "{}{}=\"{}\"", separator, additional_label_name, additional_label_value)?;
    }

    if separator == ',' {
        written += write!(w, "}}")?;
    }

    Ok(written)
}

fn write_float(w: &mut EnhancedWriter, value: f64) -> io::Result<usize> {
    match value {
        1.0 => write!(w, "1.0"),
        0.0 => write!(w, "0.0"),
        -1.0 => write!(w, "-1.0"),
        f if f.is_nan() => write!(w, "NaN"),
        f if f.is_infinite() && f.is_sign_positive() => write!(w, "+Inf"),
        f if f.is_infinite() && f.is_sign_negative() => write!(w, "-Inf"),
        _ => {
            let mut buf = ryu::Buffer::new();
            let s = buf.format(value);
            if !s.contains('.') && !s.contains('e') {
                write!(w, "{}.0", s)
            } else {
                write!(w, "{}", s)
            }
        }
    }
}

fn write_int(w: &mut EnhancedWriter, value: i64) -> io::Result<usize> {
    let mut buf = itoa::Buffer::new();
    let s = buf.format(value);
    write!(w, "{}", s)
}

fn escape_string(s: &str, escape_double_quote: bool) -> String {
    let mut escaped = String::new();
    for c in s.chars() {
        match c {
            '\\' => escaped.push_str(r"\\"),
            '\n' => escaped.push_str(r"\n"),
            '"' if escape_double_quote => escaped.push_str(r#"\""#),
            _ => escaped.push(c),
        }
    }
    escaped
}

fn metric_type_to_str(metric_type: ProtoMetricType) -> &'static str {
    match metric_type {
        ProtoMetricType::COUNTER => "counter",
        ProtoMetricType::GAUGE => "gauge",
        ProtoMetricType::SUMMARY => "summary",
        ProtoMetricType::UNTYPED => "unknown",
        ProtoMetricType::HISTOGRAM => "histogram",
        _ => "unknown",
    }
}

fn is_valid_legacy_metric_name(name: &str) -> bool {
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':')
}