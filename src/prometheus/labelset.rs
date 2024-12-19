use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelName(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelValue(String);

pub type LabelSet = HashMap<LabelName, LabelValue>;

impl LabelSet {
    pub fn validate(&self) -> Result<(), String> {
        for (ln, lv) in self {
            if !ln.is_valid() {
                return Err(format!("invalid name {}", ln));
            }
            if !lv.is_valid() {
                return Err(format!("invalid value {}", lv));
            }
        }
        Ok(())
    }

    pub fn equal(&self, other: &LabelSet) -> bool {
        self == other
    }

    pub fn before(&self, other: &LabelSet) -> bool {
        if self.len() < other.len() {
            return true;
        }
        if self.len() > other.len() {
            return false;
        }

        let mut lns: Vec<&LabelName> = self.keys().chain(other.keys()).collect();
        lns.sort();
        for ln in lns {
            match (self.get(ln), other.get(ln)) {
                (None, Some(_)) => return true,
                (Some(_), None) => return false,
                (Some(mlv), Some(olv)) => match mlv.cmp(olv) {
                    Ordering::Less => return true,
                    Ordering::Greater => return false,
                    Ordering::Equal => continue,
                },
                (None, None) => continue,
            }
        }
        false
    }

    pub fn clone(&self) -> LabelSet {
        self.clone()
    }

    pub fn merge(&self, other: &LabelSet) -> LabelSet {
        let mut result = self.clone();
        for (k, v) in other {
            result.insert(k.clone(), v.clone());
        }
        result
    }

    pub fn fingerprint(&self) -> Fingerprint {
        label_set_to_fingerprint(self)
    }

    pub fn fast_fingerprint(&self) -> Fingerprint {
        label_set_to_fast_fingerprint(self)
    }
}

impl<'de> Deserialize<'de> for LabelSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let m: HashMap<LabelName, LabelValue> = HashMap::deserialize(deserializer)?;
        for ln in m.keys() {
            if !ln.is_valid() {
                return Err(serde::de::Error::custom(format!("{} is not a valid label name", ln)));
            }
        }
        Ok(m)
    }
}

impl fmt::Display for LabelName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl LabelName {
    pub fn is_valid(&self) -> bool {
        // Assume implementation of validation logic
        true
    }
}

impl LabelValue {
    pub fn is_valid(&self) -> bool {
        // Assume implementation of validation logic
        true
    }
}