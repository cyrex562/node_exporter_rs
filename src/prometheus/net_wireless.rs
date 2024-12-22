use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::num::ParseIntError;
use std::path::Path;
use thiserror::Error;

#[derive(Debug)]
pub struct Wireless {
    pub name: String,
    pub status: u64,
    pub quality_link: i32,
    pub quality_level: i32,
    pub quality_noise: i32,
    pub discarded_nwid: i32,
    pub discarded_crypt: i32,
    pub discarded_frag: i32,
    pub discarded_retry: i32,
    pub discarded_misc: i32,
    pub missed_beacon: i32,
}

#[derive(Debug, Error)]
pub enum WirelessError {
    #[error("file read error")]
    FileReadError(#[from] io::Error),
    #[error("parse error")]
    ParseError(#[from] ParseIntError),
    #[error("invalid format: {0}")]
    InvalidFormat(String),
}

pub fn wireless<P: AsRef<Path>>(path: P) -> Result<Vec<Wireless>, WirelessError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    parse_wireless(reader)
}

fn parse_wireless<R: BufRead>(reader: R) -> Result<Vec<Wireless>, WirelessError> {
    let mut interfaces = Vec::new();
    let mut lines = reader.lines();

    // Skip the 2 header lines
    lines.next();
    lines.next();

    for line in lines {
        let line = line?;
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() != 2 {
            return Err(WirelessError::InvalidFormat(format!(
                "expected 2 parts after splitting line by ':', got {} for line {}",
                parts.len(),
                line
            )));
        }

        let name = parts[0].trim().to_string();
        let stats: Vec<&str> = parts[1].split_whitespace().collect();
        if stats.len() < 10 {
            return Err(WirelessError::InvalidFormat(format!(
                "invalid number of fields, expected 10+, got {}: {}",
                stats.len(),
                line
            )));
        }

        let status = u64::from_str_radix(stats[0], 16)?;
        let quality_link = parse_int(stats[1])?;
        let quality_level = parse_int(stats[2])?;
        let quality_noise = parse_int(stats[3])?;
        let discarded_nwid = stats[4].parse()?;
        let discarded_crypt = stats[5].parse()?;
        let discarded_frag = stats[6].parse()?;
        let discarded_retry = stats[7].parse()?;
        let discarded_misc = stats[8].parse()?;
        let missed_beacon = stats[9].parse()?;

        interfaces.push(Wireless {
            name,
            status,
            quality_link,
            quality_level,
            quality_noise,
            discarded_nwid,
            discarded_crypt,
            discarded_frag,
            discarded_retry,
            discarded_misc,
            missed_beacon,
        });
    }

    Ok(interfaces)
}

fn parse_int(s: &str) -> Result<i32, ParseIntError> {
    let s = s.trim_end_matches('.');
    s.parse::<i32>()
}
