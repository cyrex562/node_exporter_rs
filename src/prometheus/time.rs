use std::error::Error;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::num::ParseFloatError;

#[derive(Debug)]
struct NaNOrInfError;

impl fmt::Display for NaNOrInfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value is NaN or Inf")
    }
}

impl Error for NaNOrInfError {}

fn convert_to_float(value: &dyn std::any::Any) -> Result<f64, Box<dyn Error>> {
    if let Some(v) = value.downcast_ref::<f64>() {
        Ok(*v)
    } else if let Some(v) = value.downcast_ref::<String>() {
        v.parse::<f64>().map_err(|e| Box::new(e) as Box<dyn Error>)
    } else if let Some(v) = value.downcast_ref::<i32>() {
        Ok(*v as f64)
    } else if let Some(v) = value.downcast_ref::<u32>() {
        Ok(*v as f64)
    } else if let Some(v) = value.downcast_ref::<i64>() {
        Ok(*v as f64)
    } else if let Some(v) = value.downcast_ref::<u64>() {
        Ok(*v as f64)
    } else if let Some(v) = value.downcast_ref::<Duration>() {
        Ok(v.as_secs_f64())
    } else {
        Err(Box::new(fmt::Error) as Box<dyn Error>)
    }
}

fn float_to_time(value: f64) -> Result<SystemTime, Box<dyn Error>> {
    if value.is_nan() || value.is_infinite() {
        return Err(Box::new(NaNOrInfError));
    }
    let timestamp = value * 1e9;
    if timestamp > i64::MAX as f64 || timestamp < i64::MIN as f64 {
        return Err(Box::new(fmt::Error) as Box<dyn Error>);
    }
    let duration = Duration::from_nanos(timestamp as u64);
    Ok(UNIX_EPOCH + duration)
}

fn humanize_duration(value: &dyn std::any::Any) -> Result<String, Box<dyn Error>> {
    let v = convert_to_float(value)?;
    if v.is_nan() || v.is_infinite() {
        return Ok(format!("{:.4}", v));
    }
    if v == 0.0 {
        return Ok(format!("{:.4}s", v));
    }
    if v.abs() >= 1.0 {
        let sign = if v < 0.0 { "-" } else { "" };
        let mut v = v.abs();
        let duration = v as i64;
        let seconds = duration % 60;
        let minutes = (duration / 60) % 60;
        let hours = (duration / 60 / 60) % 24;
        let days = duration / 60 / 60 / 24;
        if days != 0 {
            return Ok(format!("{}{}d {}h {}m {}s", sign, days, hours, minutes, seconds));
        }
        if hours != 0 {
            return Ok(format!("{}{}h {}m {}s", sign, hours, minutes, seconds));
        }
        if minutes != 0 {
            return Ok(format!("{}{}m {}s", sign, minutes, seconds));
        }
        return Ok(format!("{}{}.4gs", sign, v));
    }
    let mut prefix = "";
    let mut v = v;
    for p in &["m", "u", "n", "p", "f", "a", "z", "y"] {
        if v.abs() >= 1.0 {
            break;
        }
        prefix = p;
        v *= 1000.0;
    }
    Ok(format!("{:.4}{}s", v, prefix))
}

fn humanize_timestamp(value: &dyn std::any::Any) -> Result<String, Box<dyn Error>> {
    let v = convert_to_float(value)?;
    match float_to_time(v) {
        Ok(tm) => Ok(format!("{:?}", tm)),
        Err(e) if e.downcast_ref::<NaNOrInfError>().is_some() => Ok(format!("{:.4}", v)),
        Err(e) => Err(e),
    }
}

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MINIMUM_TICK: Duration = Duration::from_millis(1);
const SECOND: i64 = 1_000;
const NANOS_PER_TICK: i64 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time(i64);

impl Time {
    pub fn now() -> Self {
        Self::from_unix_nano(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as i64)
    }

    pub fn from_unix(t: i64) -> Self {
        Time(t * SECOND)
    }

    pub fn from_unix_nano(t: i64) -> Self {
        Time(t / NANOS_PER_TICK)
    }

    pub fn equal(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    pub fn before(&self, other: &Self) -> bool {
        self.0 < other.0
    }

    pub fn after(&self, other: &Self) -> bool {
        self.0 > other.0
    }

    pub fn add(&self, d: Duration) -> Self {
        Time(self.0 + d.as_millis() as i64)
    }

    pub fn sub(&self, other: &Self) -> Duration {
        Duration::from_millis((self.0 - other.0) as u64)
    }

    pub fn to_system_time(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(self.0 as u64)
    }

    pub fn unix(&self) -> i64 {
        self.0 / SECOND
    }

    pub fn unix_nano(&self) -> i64 {
        self.0 * NANOS_PER_TICK
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0 as f64 / SECOND as f64)
    }
}

impl Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Time {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split('.').collect();
        match parts.len() {
            1 => {
                let v = i64::from_str(parts[0]).map_err(serde::de::Error::custom)?;
                Ok(Time(v * SECOND))
            }
            2 => {
                let mut v = i64::from_str(parts[0]).map_err(serde::de::Error::custom)? * SECOND;
                let mut frac = parts[1].to_string();
                let prec = 3 - frac.len();
                if prec < 0 {
                    frac.truncate(3);
                } else if prec > 0 {
                    frac.push_str(&"0".repeat(prec as usize));
                }
                let va = i64::from_str(&frac).map_err(serde::de::Error::custom)?;
                if parts[0].starts_with('-') && v + va > 0 {
                    Ok(Time(v + va) * -1)
                } else {
                    Ok(Time(v + va))
                }
            }
            _ => Err(serde::de::Error::custom("invalid time format")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interval {
    pub start: Time,
    pub end: Time,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CustomDuration(Duration);

impl FromStr for CustomDuration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_duration(s).map(CustomDuration).map_err(|e| e.to_string())
    }
}

impl fmt::Display for CustomDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ms = self.0.as_millis() as i64;
        if ms == 0 {
            return write!(f, "0s");
        }

        let mut result = String::new();
        let units = [
            ("y", 365 * 24 * 60 * 60 * 1000),
            ("w", 7 * 24 * 60 * 60 * 1000),
            ("d", 24 * 60 * 60 * 1000),
            ("h", 60 * 60 * 1000),
            ("m", 60 * 1000),
            ("s", 1000),
            ("ms", 1),
        ];

        for &(unit, mult) in &units {
            if ms >= mult {
                let value = ms / mult;
                ms %= mult;
                result.push_str(&format!("{}{}", value, unit));
            }
        }

        write!(f, "{}", result)
    }
}

impl Serialize for CustomDuration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for CustomDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        CustomDuration::from_str(&s).map_err(serde::de::Error::custom)
    }
}

fn parse_duration(s: &str) -> Result<Duration, &'static str> {
    if s.is_empty() {
        return Err("empty duration string");
    }

    let mut dur = 0u64;
    let mut last_unit_pos = 0;
    let mut s = s;

    while !s.is_empty() {
        let num_end = s.find(|c: char| !c.is_digit(10)).unwrap_or(s.len());
        let num_str = &s[..num_end];
        let num = num_str.parse::<u64>().map_err(|_| "invalid number")?;
        s = &s[num_end..];

        let unit_end = s.find(|c: char| c.is_digit(10)).unwrap_or(s.len());
        let unit_str = &s[..unit_end];
        s = &s[unit_end..];

        let unit = match unit_str {
            "ms" => 1,
            "s" => 1_000,
            "m" => 60_000,
            "h" => 3_600_000,
            "d" => 86_400_000,
            "w" => 604_800_000,
            "y" => 31_536_000_000,
            _ => return Err("unknown unit"),
        };

        if unit <= last_unit_pos {
            return Err("units must be in descending order");
        }
        last_unit_pos = unit;

        dur = dur.checked_add(num.checked_mul(unit).ok_or("duration overflow")?).ok_or("duration overflow")?;
    }

    Ok(Duration::from_millis(dur))
}