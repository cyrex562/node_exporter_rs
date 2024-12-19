use regex::Regex;
use serde::{Deserialize, Deserializer};
use serde::de::{self, Visitor};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
pub struct Matcher {
    name: LabelName,
    value: String,
    is_regex: bool,
}

impl<'de> Deserialize<'de> for Matcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct MatcherData {
            name: LabelName,
            value: String,
            is_regex: bool,
        }

        let data = MatcherData::deserialize(deserializer)?;
        if data.name.is_empty() {
            return Err(de::Error::custom("label name in matcher must not be empty"));
        }
        if data.is_regex {
            if Regex::new(&data.value).is_err() {
                return Err(de::Error::custom("invalid regular expression"));
            }
        }
        Ok(Matcher {
            name: data.name,
            value: data.value,
            is_regex: data.is_regex,
        })
    }
}

impl Matcher {
    pub fn validate(&self) -> Result<(), String> {
        if !self.name.is_valid() {
            return Err(format!("invalid name {}", self.name));
        }
        if self.is_regex {
            if Regex::new(&self.value).is_err() {
                return Err(format!("invalid regular expression {}", self.value));
            }
        } else if !LabelValue::new(&self.value).is_valid() || self.value.is_empty() {
            return Err(format!("invalid value {}", self.value));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct Silence {
    id: Option<u64>,
    matchers: Vec<Matcher>,
    starts_at: SystemTime,
    ends_at: SystemTime,
    created_at: Option<SystemTime>,
    created_by: String,
    comment: Option<String>,
}

impl Silence {
    pub fn validate(&self) -> Result<(), String> {
        if self.matchers.is_empty() {
            return Err("at least one matcher required".to_string());
        }
        for matcher in &self.matchers {
            matcher.validate().map_err(|e| format!("invalid matcher: {}", e))?;
        }
        if self.starts_at == UNIX_EPOCH {
            return Err("start time missing".to_string());
        }
        if self.ends_at == UNIX_EPOCH {
            return Err("end time missing".to_string());
        }
        if self.ends_at < self.starts_at {
            return Err("start time must be before end time".to_string());
        }
        if self.created_by.is_empty() {
            return Err("creator information missing".to_string());
        }
        if self.comment.as_ref().map_or(true, |c| c.is_empty()) {
            return Err("comment missing".to_string());
        }
        if self.created_at == Some(UNIX_EPOCH) {
            return Err("creation timestamp missing".to_string());
        }
        Ok(())
    }
}