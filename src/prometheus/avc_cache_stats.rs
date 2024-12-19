use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Default)]
pub struct AVCStat {
    lookups: u64,
    hits: u64,
    misses: u64,
    allocations: u64,
    reclaims: u64,
    frees: u64,
}

pub struct FS {
    selinux_path: String,
}

impl FS {
    pub fn new(selinux_path: String) -> Self {
        FS { selinux_path }
    }

    pub fn parse_avc_stats(&self) -> Result<AVCStat, io::Error> {
        let mut avc_stat = AVCStat::default();
        let file_path = Path::new(&self.selinux_path).join("avc/cache_stats");
        let file = File::open(file_path)?;
        let reader = io::BufReader::new(file);

        let mut lines = reader.lines();
        lines.next(); // Skip header

        for line in lines {
            let line = line?;
            let avc_values: Vec<&str> = line.split_whitespace().collect();

            if avc_values.len() != 6 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("invalid AVC stat line: {}", line)));
            }

            avc_stat.lookups += u64::from_str(avc_values[0]).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "could not parse expected integer value for lookups"))?;
            avc_stat.hits += u64::from_str(avc_values[1]).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "could not parse expected integer value for hits"))?;
            avc_stat.misses += u64::from_str(avc_values[2]).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "could not parse expected integer value for misses"))?;
            avc_stat.allocations += u64::from_str(avc_values[3]).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "could not parse expected integer value for allocations"))?;
            avc_stat.reclaims += u64::from_str(avc_values[4]).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "could not parse expected integer value for reclaims"))?;
            avc_stat.frees += u64::from_str(avc_values[5]).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "could not parse expected integer value for frees"))?;
        }

        Ok(avc_stat)
    }
}