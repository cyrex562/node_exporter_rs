use prometheus_client::encoding::protobuf::Bucket as ProtoBucket;
use prometheus_client::encoding::protobuf::Counter as ProtoCounter;
use prometheus_client::encoding::protobuf::Exemplar as ProtoExemplar;
use prometheus_client::encoding::protobuf::Histogram as ProtoHistogram;
use prometheus_client::encoding::protobuf::LabelPair;
use prometheus_client::encoding::protobuf::Metric as ProtoMetric;
use prometheus_client::encoding::protobuf::MetricFamily;
use prost::Message;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub trait Metric {
    fn desc(&self) -> &Desc;
    fn write(&self, metric: &mut ProtoMetric) -> Result<(), Box<dyn Error>>;
}

pub struct Opts {
    pub namespace: String,
    pub subsystem: String,
    pub name: String,
    pub help: String,
    pub const_labels: HashMap<String, String>,
    pub now: fn() -> SystemTime,
}

impl Opts {
    pub fn new(namespace: &str, subsystem: &str, name: &str, help: &str) -> Self {
        Opts {
            namespace: namespace.to_string(),
            subsystem: subsystem.to_string(),
            name: name.to_string(),
            help: help.to_string(),
            const_labels: HashMap::new(),
            now: SystemTime::now,
        }
    }
}

pub fn build_fq_name(namespace: &str, subsystem: &str, name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }

    let mut fq_name = String::new();
    if !namespace.is_empty() {
        fq_name.push_str(namespace);
        fq_name.push('_');
    }
    if !subsystem.is_empty() {
        fq_name.push_str(subsystem);
        fq_name.push('_');
    }
    fq_name.push_str(name);
    fq_name
}

pub struct InvalidMetric {
    desc: Desc,
    err: Box<dyn Error>,
}

impl InvalidMetric {
    pub fn new(desc: Desc, err: Box<dyn Error>) -> Self {
        InvalidMetric { desc, err }
    }
}

impl Metric for InvalidMetric {
    fn desc(&self) -> &Desc {
        &self.desc
    }

    fn write(&self, _metric: &mut ProtoMetric) -> Result<(), Box<dyn Error>> {
        Err(self.err.clone())
    }
}

pub struct TimestampedMetric {
    metric: Box<dyn Metric>,
    t: SystemTime,
}

impl TimestampedMetric {
    pub fn new(t: SystemTime, metric: Box<dyn Metric>) -> Self {
        TimestampedMetric { metric, t }
    }
}

impl Metric for TimestampedMetric {
    fn desc(&self) -> &Desc {
        self.metric.desc()
    }

    fn write(&self, metric: &mut ProtoMetric) -> Result<(), Box<dyn Error>> {
        self.metric.write(metric)?;
        let duration = self.t.duration_since(UNIX_EPOCH).unwrap();
        metric.timestamp_ms = Some(duration.as_millis() as i64);
        Ok(())
    }
}

pub struct WithExemplarsMetric {
    metric: Box<dyn Metric>,
    exemplars: Vec<ProtoExemplar>,
}

impl WithExemplarsMetric {
    pub fn new(metric: Box<dyn Metric>, exemplars: Vec<ProtoExemplar>) -> Self {
        WithExemplarsMetric { metric, exemplars }
    }
}

impl Metric for WithExemplarsMetric {
    fn desc(&self) -> &Desc {
        self.metric.desc()
    }

    fn write(&self, metric: &mut ProtoMetric) -> Result<(), Box<dyn Error>> {
        self.metric.write(metric)?;

        if let Some(counter) = &mut metric.counter {
            counter.exemplar = Some(self.exemplars.last().unwrap().clone());
        } else if let Some(histogram) = &mut metric.histogram {
            for exemplar in &self.exemplars {
                let bucket_index = histogram
                    .bucket
                    .iter()
                    .position(|bucket| bucket.upper_bound >= exemplar.value)
                    .unwrap_or(histogram.bucket.len());
                if bucket_index < histogram.bucket.len() {
                    histogram.bucket[bucket_index].exemplar = Some(exemplar.clone());
                } else {
                    let bucket = ProtoBucket {
                        cumulative_count: Some(histogram.sample_count.unwrap_or(0)),
                        upper_bound: Some(f64::INFINITY),
                        exemplar: Some(exemplar.clone()),
                    };
                    histogram.bucket.push(bucket);
                }
            }
        } else {
            return Err(Box::new(fmt::Error::new(
                fmt::Error,
                "cannot inject exemplar into Gauge, Summary or Untyped",
            )));
        }

        Ok(())
    }
}

pub struct Exemplar {
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: Option<SystemTime>,
}

impl Exemplar {
    pub fn new(value: f64, labels: HashMap<String, String>, timestamp: Option<SystemTime>) -> Self {
        Exemplar {
            value,
            labels,
            timestamp,
        }
    }
}

pub fn new_metric_with_exemplars(
    metric: Box<dyn Metric>,
    exemplars: Vec<Exemplar>,
) -> Result<Box<dyn Metric>, Box<dyn Error>> {
    if exemplars.is_empty() {
        return Err(Box::new(fmt::Error::new(
            fmt::Error,
            "no exemplar was passed for new_metric_with_exemplars",
        )));
    }

    let now = SystemTime::now();
    let proto_exemplars: Result<Vec<ProtoExemplar>, Box<dyn Error>> = exemplars
        .into_iter()
        .map(|e| {
            let timestamp = e.timestamp.unwrap_or(now);
            let duration = timestamp.duration_since(UNIX_EPOCH).unwrap();
            let proto_exemplar = ProtoExemplar {
                value: Some(e.value),
                timestamp: Some(duration.as_millis() as i64),
                ..Default::default()
            };
            Ok(proto_exemplar)
        })
        .collect();

    Ok(Box::new(WithExemplarsMetric::new(metric, proto_exemplars?)))
}

pub fn must_new_metric_with_exemplars(
    metric: Box<dyn Metric>,
    exemplars: Vec<Exemplar>,
) -> Box<dyn Metric> {
    new_metric_with_exemplars(metric, exemplars).unwrap()
}

use std::cmp::Ordering;
use std::slice;
use std::vec::Vec;

#[derive(Debug)]
struct LabelPair {
    name: String,
    value: String,
}

impl LabelPair {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug)]
struct Metric {
    label: Vec<LabelPair>,
    timestamp_ms: Option<i64>,
}

impl Metric {
    fn get_timestamp_ms(&self) -> Option<i64> {
        self.timestamp_ms
    }
}

#[derive(Debug)]
struct MetricFamily {
    metric: Vec<Metric>,
}

struct LabelPairSorter<'a>(&'a mut [LabelPair]);

impl<'a> LabelPairSorter<'a> {
    fn sort(&mut self) {
        self.0.sort_by(|a, b| a.get_name().cmp(b.get_name()));
    }
}

struct MetricSorter<'a>(&'a mut [Metric]);

impl<'a> MetricSorter<'a> {
    fn sort(&mut self) {
        self.0.sort_by(|a, b| {
            if a.label.len() != b.label.len() {
                return a.label.len().cmp(&b.label.len());
            }
            for (lp_a, lp_b) in a.label.iter().zip(&b.label) {
                let vi = lp_a.get_value();
                let vj = lp_b.get_value();
                if vi != vj {
                    return vi.cmp(vj);
                }
            }
            match (a.get_timestamp_ms(), b.get_timestamp_ms()) {
                (Some(ts_a), Some(ts_b)) => ts_a.cmp(&ts_b),
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (None, None) => Ordering::Equal,
            }
        });
    }
}

fn normalize_metric_families(
    metric_families_by_name: &mut HashMap<String, MetricFamily>,
) -> Vec<MetricFamily> {
    for mf in metric_families_by_name.values_mut() {
        MetricSorter(&mut mf.metric).sort();
    }
    let mut names: Vec<String> = metric_families_by_name
        .iter()
        .filter_map(|(name, mf)| {
            if !mf.metric.is_empty() {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names
        .into_iter()
        .filter_map(|name| metric_families_by_name.remove(&name))
        .collect()
}

fn main() {
    // Example usage
    let mut metric_families_by_name = HashMap::new();
    metric_families_by_name.insert(
        "family1".to_string(),
        MetricFamily {
            metric: vec![
                Metric {
                    label: vec![
                        LabelPair {
                            name: "label1".to_string(),
                            value: "value1".to_string(),
                        },
                        LabelPair {
                            name: "label2".to_string(),
                            value: "value2".to_string(),
                        },
                    ],
                    timestamp_ms: Some(1000),
                },
                Metric {
                    label: vec![
                        LabelPair {
                            name: "label1".to_string(),
                            value: "value1".to_string(),
                        },
                        LabelPair {
                            name: "label2".to_string(),
                            value: "value3".to_string(),
                        },
                    ],
                    timestamp_ms: Some(2000),
                },
            ],
        },
    );

    let normalized = normalize_metric_families(&mut metric_families_by_name);
    println!("{:?}", normalized);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    GaugeHistogram,
    Summary,
    Info,
    StateSet,
    Unknown,
}

impl MetricType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricType::Counter => "counter",
            MetricType::Gauge => "gauge",
            MetricType::Histogram => "histogram",
            MetricType::GaugeHistogram => "gaugehistogram",
            MetricType::Summary => "summary",
            MetricType::Info => "info",
            MetricType::StateSet => "stateset",
            MetricType::Unknown => "unknown",
        }
    }
}