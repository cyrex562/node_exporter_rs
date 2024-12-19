use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    metric: Metric,
    value: SampleValue,
    timestamp: Time,
    histogram: Option<SampleHistogram>,
}

impl Sample {
    pub fn equal(&self, other: &Self) -> bool {
        self.metric == other.metric
            && self.timestamp == other.timestamp
            && self.histogram.as_ref().map_or(true, |h| h.equal(other.histogram.as_ref().unwrap()))
            && self.value.equal(&other.value)
    }
}

impl fmt::Display for Sample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(histogram) = &self.histogram {
            write!(
                f,
                "{} => {}",
                self.metric,
                SampleHistogramPair {
                    timestamp: self.timestamp,
                    histogram
                }
            )
        } else {
            write!(
                f,
                "{} => {}",
                self.metric,
                SamplePair {
                    timestamp: self.timestamp,
                    value: self.value
                }
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SampleStream {
    metric: Metric,
    values: Vec<SamplePair>,
    histograms: Vec<SampleHistogramPair>,
}

impl fmt::Display for SampleStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut vals: Vec<String> = self.values.iter().map(|v| v.to_string()).collect();
        vals.extend(self.histograms.iter().map(|h| h.to_string()));
        write!(f, "{} =>\n{}", self.metric, vals.join("\n"))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scalar {
    value: SampleValue,
    timestamp: Time,
}

impl fmt::Display for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "scalar: {} @[{}]", self.value, self.timestamp)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StringValue {
    value: String,
    timestamp: Time,
}

impl fmt::Display for StringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub type Vector = Vec<Sample>;

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries: Vec<String> = self.iter().map(|s| s.to_string()).collect();
        write!(f, "{}", entries.join("\n"))
    }
}

impl Vector {
    pub fn len(&self) -> usize {
        self.len()
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.swap(i, j)
    }

    pub fn less(&self, i: usize, j: usize) -> bool {
        self[i].metric < self[j].metric || (self[i].metric == self[j].metric && self[i].timestamp < self[j].timestamp)
    }

    pub fn equal(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other).all(|(a, b)| a.equal(b))
    }
}

pub type Matrix = Vec<SampleStream>;

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut mat_cp = self.clone();
        mat_cp.sort_by(|a, b| a.metric.cmp(&b.metric));
        let strs: Vec<String> = mat_cp.iter().map(|ss| ss.to_string()).collect();
        write!(f, "{}", strs.join("\n"))
    }
}

impl Matrix {
    pub fn len(&self) -> usize {
        self.len()
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.swap(i, j)
    }

    pub fn less(&self, i: usize, j: usize) -> bool {
        self[i].metric < self[j].metric
    }
}