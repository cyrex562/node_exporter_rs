use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertStatus {
    Firing,
    Resolved,
}

#[derive(Debug, Clone)]
pub struct Alert {
    labels: HashMap<String, String>,
    annotations: HashMap<String, String>,
    starts_at: Option<SystemTime>,
    ends_at: Option<SystemTime>,
    generator_url: String,
}

impl Alert {
    pub fn name(&self) -> Option<&str> {
        self.labels.get("alertname").map(|s| s.as_str())
    }

    pub fn fingerprint(&self) -> u64 {
        // Assuming a function `fingerprint` exists that generates a unique hash for the label set
        fingerprint(&self.labels)
    }

    pub fn resolved(&self) -> bool {
        self.resolved_at(SystemTime::now())
    }

    pub fn resolved_at(&self, ts: SystemTime) -> bool {
        if let Some(ends_at) = self.ends_at {
            return ends_at <= ts;
        }
        false
    }

    pub fn status(&self) -> AlertStatus {
        self.status_at(SystemTime::now())
    }

    pub fn status_at(&self, ts: SystemTime) -> AlertStatus {
        if self.resolved_at(ts) {
            AlertStatus::Resolved
        } else {
            AlertStatus::Firing
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.starts_at.is_none() {
            return Err("start time missing".to_string());
        }
        if let (Some(starts_at), Some(ends_at)) = (self.starts_at, self.ends_at) {
            if ends_at < starts_at {
                return Err("start time must be before end time".to_string());
            }
        }
        // Assuming `validate_labels` and `validate_annotations` are implemented elsewhere
        validate_labels(&self.labels)?;
        if self.labels.is_empty() {
            return Err("at least one label pair required".to_string());
        }
        validate_annotations(&self.annotations)?;
        Ok(())
    }
}

impl fmt::Display for Alert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.resolved() { "resolved" } else { "active" };
        write!(f, "{}[{}][{}]", self.name().unwrap_or(""), self.fingerprint(), status)
    }
}

#[derive(Debug, Clone)]
pub struct Alerts(Vec<Alert>);

impl Alerts {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.0.swap(i, j);
    }

    pub fn has_firing(&self) -> bool {
        self.0.iter().any(|a| !a.resolved())
    }

    pub fn has_firing_at(&self, ts: SystemTime) -> bool {
        self.0.iter().any(|a| !a.resolved_at(ts))
    }

    pub fn status(&self) -> AlertStatus {
        if self.has_firing() {
            AlertStatus::Firing
        } else {
            AlertStatus::Resolved
        }
    }

    pub fn status_at(&self, ts: SystemTime) -> AlertStatus {
        if self.has_firing_at(ts) {
            AlertStatus::Firing
        } else {
            AlertStatus::Resolved
        }
    }
}

impl Ord for Alert {
    fn cmp(&self, other: &Self) -> Ordering {
        self.starts_at.cmp(&other.starts_at)
            .then_with(|| self.ends_at.cmp(&other.ends_at))
            .then_with(|| self.fingerprint().cmp(&other.fingerprint()))
    }
}

impl PartialOrd for Alert {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Alert {
    fn eq(&self, other: &Self) -> bool {
        self.starts_at == other.starts_at && self.ends_at == other.ends_at && self.fingerprint() == other.fingerprint()
    }
}

impl Eq for Alert {}

