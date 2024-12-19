use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SampleValue(f64);

impl SampleValue {
    pub fn equal(&self, other: &Self) -> bool {
        self.0 == other.0 || (self.0.is_nan() && other.0.is_nan())
    }
}

impl fmt::Display for SampleValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for SampleValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SampleValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let f = f64::from_str(&s).map_err(serde::de::Error::custom)?;
        Ok(SampleValue(f))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SamplePair {
    pub timestamp: Time,
    pub value: SampleValue,
}

impl SamplePair {
    pub fn equal(&self, other: &Self) -> bool {
        self.value.equal(&other.value) && self.timestamp.equal(&other.timestamp)
    }
}

impl fmt::Display for SamplePair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ [{}]", self.value, self.timestamp)
    }
}

impl Serialize for SamplePair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let t = json!(self.timestamp);
        let v = json!(self.value);
        serializer.serialize_str(&format!("[{},{}]", t, v))
    }
}

impl<'de> Deserialize<'de> for SamplePair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Vec<serde_json::Value> = Vec::deserialize(deserializer)?;
        if v.len() != 2 {
            return Err(serde::de::Error::custom("invalid SamplePair format"));
        }
        let timestamp = Time::deserialize(v[0].clone()).map_err(serde::de::Error::custom)?;
        let value = SampleValue::deserialize(v[1].clone()).map_err(serde::de::Error::custom)?;
        Ok(SamplePair { timestamp, value })
    }
}

pub const ZERO_SAMPLE_PAIR: SamplePair = SamplePair {
    timestamp: Time::earliest(),
    value: SampleValue(0.0),
};