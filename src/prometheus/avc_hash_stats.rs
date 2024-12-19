use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Default)]
pub struct AVCHashStat {
    entries: u64,
    buckets_used: u64,
    buckets_available: u64,
    longest_chain: u64,
}

pub struct FS {
    selinux_path: String,
}

impl FS {
    pub fn new(selinux_path: String) -> Self {
        FS { selinux_path }
    }

    pub fn parse_avc_hash_stats(&self) -> Result<AVCHashStat, io::Error> {
        let mut avc_hash_stat = AVCHashStat::default();
        let file_path = Path::new(&self.selinux_path).join("avc/hash_stats");
        let file = File::open(file_path)?;
        let reader = io::BufReader::new(file);

        let mut lines = reader.lines();

        if let Some(line) = lines.next() {
            let entries_value = line?.trim_start_matches("entries: ").to_string();
            avc_hash_stat.entries = u64::from_str(&entries_value).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "could not parse expected integer value for hash entries"))?;
        }

        if let Some(line) = lines.next() {
            let buckets_values: Vec<&str> = line?.split("buckets used: ").collect();
            let buckets_values_tuple: Vec<&str> = buckets_values[1].split('/').collect();
            avc_hash_stat.buckets_used = u64::from_str(buckets_values_tuple[0]).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "could not parse expected integer value for hash buckets used"))?;
            avc_hash_stat.buckets_available = u64::from_str(buckets_values_tuple[1]).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "could not parse expected integer value for hash buckets available"))?;
        }

        if let Some(line) = lines.next() {
            let longest_chain_value = line?.trim_start_matches("longest chain: ").to_string();
            avc_hash_stat.longest_chain = u64::from_str(&longest_chain_value).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "could not parse expected integer value for hash longest chain"))?;
        }

        Ok(avc_hash_stat)
    }
}