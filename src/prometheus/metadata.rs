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