use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Fingerprint(u64);

impl FromStr for Fingerprint {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = u64::from_str_radix(s, 16)?;
        Ok(Fingerprint(num))
    }
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

pub type Fingerprints = Vec<Fingerprint>;

impl Fingerprints {
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

pub type FingerprintSet = HashSet<Fingerprint>;

impl FingerprintSet {
    pub fn equal(&self, other: &FingerprintSet) -> bool {
        self == other
    }

    pub fn intersection(&self, other: &FingerprintSet) -> FingerprintSet {
        self.intersection(other).cloned().collect()
    }
}