use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatString(f64);

impl fmt::Display for FloatString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for FloatString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for FloatString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let f = f64::from_str(&s).map_err(serde::de::Error::custom)?;
        Ok(FloatString(f))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistogramBucket {
    boundaries: i32,
    lower: FloatString,
    upper: FloatString,
    count: FloatString,
}

impl Serialize for HistogramBucket {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let b = json!(self.boundaries);
        let l = json!(self.lower);
        let u = json!(self.upper);
        let c = json!(self.count);
        serializer.serialize_str(&format!("[{},{},{},{}]", b, l, u, c))
    }
}

impl<'de> Deserialize<'de> for HistogramBucket {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Vec<serde_json::Value> = Vec::deserialize(deserializer)?;
        if v.len() != 4 {
            return Err(serde::de::Error::custom("invalid HistogramBucket format"));
        }
        let boundaries = i32::deserialize(v[0].clone()).map_err(serde::de::Error::custom)?;
        let lower = FloatString::deserialize(v[1].clone()).map_err(serde::de::Error::custom)?;
        let upper = FloatString::deserialize(v[2].clone()).map_err(serde::de::Error::custom)?;
        let count = FloatString::deserialize(v[3].clone()).map_err(serde::de::Error::custom)?;
        Ok(HistogramBucket {
            boundaries,
            lower,
            upper,
            count,
        })
    }
}

impl HistogramBucket {
    pub fn equal(&self, other: &Self) -> bool {
        self.boundaries == other.boundaries
            && self.lower == other.lower
            && self.upper == other.upper
            && self.count == other.count
    }
}

impl fmt::Display for HistogramBucket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lower_inclusive = self.boundaries == 1 || self.boundaries == 3;
        let upper_inclusive = self.boundaries == 0 || self.boundaries == 3;
        let lower_bracket = if lower_inclusive { '[' } else { '(' };
        let upper_bracket = if upper_inclusive { ']' } else { ')' };
        write!(
            f,
            "{}{},{}{}:{}",
            lower_bracket, self.lower, self.upper, upper_bracket, self.count
        )
    }
}

pub type HistogramBuckets = Vec<HistogramBucket>;

impl HistogramBuckets {
    pub fn equal(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (a, b) in self.iter().zip(other.iter()) {
            if !a.equal(b) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SampleHistogram {
    count: FloatString,
    sum: FloatString,
    buckets: HistogramBuckets,
}

impl fmt::Display for SampleHistogram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Count: {}, Sum: {}, Buckets: {:?}", self.count, self.sum, self.buckets)
    }
}

impl SampleHistogram {
    pub fn equal(&self, other: &Self) -> bool {
        self.count == other.count && self.sum == other.sum && self.buckets.equal(&other.buckets)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SampleHistogramPair {
    timestamp: Time,
    histogram: SampleHistogram,
}

impl Serialize for SampleHistogramPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let t = json!(self.timestamp);
        let v = json!(self.histogram);
        serializer.serialize_str(&format!("[{},{}]", t, v))
    }
}

impl<'de> Deserialize<'de> for SampleHistogramPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Vec<serde_json::Value> = Vec::deserialize(deserializer)?;
        if v.len() != 2 {
            return Err(serde::de::Error::custom("invalid SampleHistogramPair format"));
        }
        let timestamp = Time::deserialize(v[0].clone()).map_err(serde::de::Error::custom)?;
        let histogram = SampleHistogram::deserialize(v[1].clone()).map_err(serde::de::Error::custom)?;
        Ok(SampleHistogramPair { timestamp, histogram })
    }
}

impl fmt::Display for SampleHistogramPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ [{}]", self.histogram, self.timestamp)
    }
}

impl SampleHistogramPair {
    pub fn equal(&self, other: &Self) -> bool {
        self.histogram.equal(&other.histogram) && self.timestamp.equal(&other.timestamp)
    }
}