
use std::collections::HashMap;
const _: () = {
    // Verify that this generated code is sufficiently up-to-date.
    const ENFORCE_VERSION: i32 = 20 - MIN_VERSION;
    // Verify that runtime/protoimpl is sufficiently up-to-date.
    const ENFORCE_RUNTIME_VERSION: i32 = MAX_VERSION - 20;
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
    Counter = 0,
    Gauge = 1,
    Summary = 2,
    Untyped = 3,
    Histogram = 4,
}



lazy_static::lazy_static! {
    pub static ref METRIC_TYPE_NAME: HashMap<i32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(0, "COUNTER");
        m.insert(1, "GAUGE");
        m.insert(2, "SUMMARY");
        m.insert(3, "UNTYPED");
        m.insert(4, "HISTOGRAM");
        m.insert(5, "GAUGE_HISTOGRAM");
        m
    };

    pub static ref METRIC_TYPE_VALUE: HashMap<&'static str, i32> = {
        let mut m = HashMap::new();
        m.insert("COUNTER", 0);
        m.insert("GAUGE", 1);
        m.insert("SUMMARY", 2);
        m.insert("UNTYPED", 3);
        m.insert("HISTOGRAM", 4);
        m.insert("GAUGE_HISTOGRAM", 5);
        m
    };
}

impl MetricType {
    pub fn enum_ref(&self) -> &MetricType {
        self
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            MetricType::Counter => "COUNTER",
            MetricType::Gauge => "GAUGE",
            MetricType::Summary => "SUMMARY",
            MetricType::Untyped => "UNTYPED",
            MetricType::Histogram => "HISTOGRAM",
            MetricType::GaugeHistogram => "GAUGE_HISTOGRAM",
        }
    }

    pub fn descriptor(&self) -> &'static str {
        "MetricType"
    }

    pub fn number(&self) -> i32 {
        *self as i32
    }

    pub fn from_number(num: i32) -> Option<MetricType> {
        match num {
            0 => Some(MetricType::Counter),
            1 => Some(MetricType::Gauge),
            2 => Some(MetricType::Summary),
            3 => Some(MetricType::Untyped),
            4 => Some(MetricType::Histogram),
            5 => Some(MetricType::GaugeHistogram),
            _ => None,
        }
    }
}

// Deprecated: Do not use.
impl serde::Deserialize for MetricType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let num: i32 = serde::Deserialize::deserialize(deserializer)?;
        MetricType::from_number(num).ok_or_else(|| serde::de::Error::custom("invalid MetricType"))
    }
}

// Deprecated: Use MetricType::descriptor instead.
impl MetricType {
    pub fn enum_descriptor() -> (&'static [u8], [usize; 1]) {
        (b"MetricType", [0])
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct LabelPair {
    #[prost(string, optional, tag = "1")]
    pub name: Option<String>,
    #[prost(string, optional, tag = "2")]
    pub value: Option<String>,
}

impl LabelPair {
    pub fn reset(&mut self) {
        *self = LabelPair::default();
    }

    pub fn get_name(&self) -> &str {
        self.name.as_deref().unwrap_or("")
    }

    pub fn get_value(&self) -> &str {
        self.value.as_deref().unwrap_or("")
    }
}

impl std::fmt::Debug for LabelPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LabelPair {{ name: {:?}, value: {:?} }}", self.name, self.value)
    }
}

impl std::fmt::Display for LabelPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LabelPair {{ name: {:?}, value: {:?} }}", self.name, self.value)
    }
}

impl Default for LabelPair {
    fn default() -> Self {
        LabelPair {
            name: None,
            value: None,
        }
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct Gauge {
    #[prost(double, optional, tag = "1")]
    pub value: Option<f64>,
}

impl Gauge {
    pub fn reset(&mut self) {
        *self = Gauge::default();
    }

    pub fn get_value(&self) -> f64 {
        self.value.unwrap_or(0.0)
    }
}

impl std::fmt::Debug for Gauge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Gauge {{ value: {:?} }}", self.value)
    }
}

impl std::fmt::Display for Gauge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Gauge {{ value: {:?} }}", self.value)
    }
}

impl Default for Gauge {
    fn default() -> Self {
        Gauge {
            value: None,
        }
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct Counter {
    #[prost(double, optional, tag = "1")]
    pub value: Option<f64>,
    #[prost(message, optional, tag = "2")]
    pub exemplar: Option<Exemplar>,
    #[prost(message, optional, tag = "3", rename = "created_timestamp")]
    pub created_timestamp: Option<Timestamp>,
}

impl Counter {
    pub fn reset(&mut self) {
        *self = Counter::default();
    }

    pub fn get_value(&self) -> f64 {
        self.value.unwrap_or(0.0)
    }

    pub fn get_exemplar(&self) -> Option<&Exemplar> {
        self.exemplar.as_ref()
    }

    pub fn get_created_timestamp(&self) -> Option<&Timestamp> {
        self.created_timestamp.as_ref()
    }
}

impl std::fmt::Debug for Counter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Counter {{ value: {:?}, exemplar: {:?}, created_timestamp: {:?} }}", self.value, self.exemplar, self.created_timestamp)
    }
}

impl std::fmt::Display for Counter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Counter {{ value: {:?}, exemplar: {:?}, created_timestamp: {:?} }}", self.value, self.exemplar, self.created_timestamp)
    }
}

impl Default for Counter {
    fn default() -> Self {
        Counter {
            value: None,
            exemplar: None,
            created_timestamp: None,
        }
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct Quantile {
    #[prost(double, optional, tag = "1")]
    pub quantile: Option<f64>,
    #[prost(double, optional, tag = "2")]
    pub value: Option<f64>,
}

impl Quantile {
    pub fn reset(&mut self) {
        *self = Quantile::default();
    }

    pub fn get_quantile(&self) -> f64 {
        self.quantile.unwrap_or(0.0)
    }

    pub fn get_value(&self) -> f64 {
        self.value.unwrap_or(0.0)
    }
}

impl std::fmt::Debug for Quantile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Quantile {{ quantile: {:?}, value: {:?} }}", self.quantile, self.value)
    }
}

impl std::fmt::Display for Quantile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Quantile {{ quantile: {:?}, value: {:?} }}", self.quantile, self.value)
    }
}

impl Default for Quantile {
    fn default() -> Self {
        Quantile {
            quantile: None,
            value: None,
        }
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct Summary {
    #[prost(uint64, optional, tag = "1")]
    pub sample_count: Option<u64>,
    #[prost(double, optional, tag = "2")]
    pub sample_sum: Option<f64>,
    #[prost(message, repeated, tag = "3")]
    pub quantile: Vec<Quantile>,
    #[prost(message, optional, tag = "4", rename = "created_timestamp")]
    pub created_timestamp: Option<Timestamp>,
}

impl Summary {
    pub fn reset(&mut self) {
        *self = Summary::default();
    }

    pub fn get_sample_count(&self) -> u64 {
        self.sample_count.unwrap_or(0)
    }

    pub fn get_sample_sum(&self) -> f64 {
        self.sample_sum.unwrap_or(0.0)
    }

    pub fn get_quantile(&self) -> &Vec<Quantile> {
        &self.quantile
    }

    pub fn get_created_timestamp(&self) -> Option<&Timestamp> {
        self.created_timestamp.as_ref()
    }
}

impl std::fmt::Debug for Summary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Summary {{ sample_count: {:?}, sample_sum: {:?}, quantile: {:?}, created_timestamp: {:?} }}", self.sample_count, self.sample_sum, self.quantile, self.created_timestamp)
    }
}

impl std::fmt::Display for Summary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Summary {{ sample_count: {:?}, sample_sum: {:?}, quantile: {:?}, created_timestamp: {:?} }}", self.sample_count, self.sample_sum, self.quantile, self.created_timestamp)
    }
}

impl Default for Summary {
    fn default() -> Self {
        Summary {
            sample_count: None,
            sample_sum: None,
            quantile: Vec::new(),
            created_timestamp: None,
        }
    }
}