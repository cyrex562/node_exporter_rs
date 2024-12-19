use regex::Regex;
use serde::{Deserialize, Deserializer};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;
use unicode_segmentation::UnicodeSegmentation;

const ALERT_NAME_LABEL: &str = "alertname";
const EXPORTED_LABEL_PREFIX: &str = "exported_";
const METRIC_NAME_LABEL: &str = "__name__";
const SCHEME_LABEL: &str = "__scheme__";
const ADDRESS_LABEL: &str = "__address__";
const METRICS_PATH_LABEL: &str = "__metrics_path__";
const SCRAPE_INTERVAL_LABEL: &str = "__scrape_interval__";
const SCRAPE_TIMEOUT_LABEL: &str = "__scrape_timeout__";
const RESERVED_LABEL_PREFIX: &str = "__";
const META_LABEL_PREFIX: &str = "__meta_";
const TMP_LABEL_PREFIX: &str = "__tmp_";
const PARAM_LABEL_PREFIX: &str = "__param_";
const JOB_LABEL: &str = "job";
const INSTANCE_LABEL: &str = "instance";
const BUCKET_LABEL: &str = "le";
const QUANTILE_LABEL: &str = "quantile";

lazy_static::lazy_static! {
    static ref LABEL_NAME_RE: Regex = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LabelName(String);

impl LabelName {
    pub fn is_valid(&self) -> bool {
        !self.0.is_empty() && LABEL_NAME_RE.is_match(&self.0)
    }

    pub fn is_valid_legacy(&self) -> bool {
        if self.0.is_empty() {
            return false;
        }
        for (i, c) in self.0.chars().enumerate() {
            if !((c.is_ascii_alphabetic() || c == '_') || (c.is_ascii_digit() && i > 0)) {
                return false;
            }
        }
        true
    }
}

impl<'de> Deserialize<'de> for LabelName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let ln = LabelName(s);
        if !ln.is_valid() {
            return Err(serde::de::Error::custom(format!("{} is not a valid label name", ln.0)));
        }
        Ok(ln)
    }
}

impl fmt::Display for LabelName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LabelValue(String);

impl LabelValue {
    pub fn is_valid(&self) -> bool {
        self.0.is_grapheme(true)
    }
}

impl<'de> Deserialize<'de> for LabelValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let lv = LabelValue(s);
        if !lv.is_valid() {
            return Err(serde::de::Error::custom(format!("{} is not a valid label value", lv.0)));
        }
        Ok(lv)
    }
}

impl fmt::Display for LabelValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type LabelNames = Vec<LabelName>;

impl LabelNames {
    pub fn len(&self) -> usize {
        self.len()
    }

    pub fn less(&self, i: usize, j: usize) -> bool {
        self[i] < self[j]
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.swap(i, j)
    }

    pub fn to_string(&self) -> String {
        self.iter().map(|ln| ln.to_string()).collect::<Vec<_>>().join(", ")
    }
}

pub type LabelValues = Vec<LabelValue>;

impl LabelValues {
    pub fn len(&self) -> usize {
        self.len()
    }

    pub fn less(&self, i: usize, j: usize) -> bool {
        self[i] < self[j]
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.swap(i, j)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelPair {
    pub name: LabelName,
    pub value: LabelValue,
}

pub type LabelPairs = Vec<LabelPair>;

impl LabelPairs {
    pub fn len(&self) -> usize {
        self.len()
    }

    pub fn less(&self, i: usize, j: usize) -> bool {
        match self[i].name.cmp(&self[j].name) {
            Ordering::Equal => self[i].value < self[j].value,
            other => other == Ordering::Less,
        }
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.swap(i, j)
    }
}