use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::num::ParseIntError;
use std::path::Path;
use thiserror::Error;

#[derive(Debug)]
pub struct SoftnetStat {
    pub processed: u32,
    pub dropped: u32,
    pub time_squeezed: u32,
    pub cpu_collision: u32,
    pub received_rps: u32,
    pub flow_limit_count: u32,
    pub softnet_backlog_len: u32,
    pub index: u32,
    pub width: usize,
}

#[derive(Debug, Error)]
pub enum SoftnetStatError {
    #[error("file read error")]
    FileReadError(#[from] io::Error),
    #[error("parse error")]
    ParseError(#[from] ParseIntError),
    #[error("invalid number of columns: {0}")]
    InvalidNumberOfColumns(usize),
}

pub fn net_softnet_stat<P: AsRef<Path>>(path: P) -> Result<Vec<SoftnetStat>, SoftnetStatError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    parse_softnet(reader)
}

fn parse_softnet<R: Read>(reader: R) -> Result<Vec<SoftnetStat>, SoftnetStatError> {
    const MIN_COLUMNS: usize = 9;
    let mut stats = Vec::new();
    let mut lines = BufReader::new(reader).lines();
    let mut cpu_index = 0;

    while let Some(line) = lines.next() {
        let line = line?;
        let columns: Vec<&str> = line.split_whitespace().collect();
        let width = columns.len();

        if width < MIN_COLUMNS {
            return Err(SoftnetStatError::InvalidNumberOfColumns(width));
        }

        let mut softnet_stat = SoftnetStat {
            processed: 0,
            dropped: 0,
            time_squeezed: 0,
            cpu_collision: 0,
            received_rps: 0,
            flow_limit_count: 0,
            softnet_backlog_len: 0,
            index: cpu_index,
            width,
        };

        if width >= MIN_COLUMNS {
            let us = parse_hex_u32s(&columns[0..9])?;
            softnet_stat.processed = us[0];
            softnet_stat.dropped = us[1];
            softnet_stat.time_squeezed = us[2];
            softnet_stat.cpu_collision = us[8];
        }

        if width >= 10 {
            let us = parse_hex_u32s(&columns[9..10])?;
            softnet_stat.received_rps = us[0];
        }

        if width >= 11 {
            let us = parse_hex_u32s(&columns[10..11])?;
            softnet_stat.flow_limit_count = us[0];
        }

        if width >= 13 {
            let us = parse_hex_u32s(&columns[11..13])?;
            softnet_stat.softnet_backlog_len = us[0];
            softnet_stat.index = us[1];
        }

        stats.push(softnet_stat);
        cpu_index += 1;
    }

    Ok(stats)
}

fn parse_hex_u32s(ss: &[&str]) -> Result<Vec<u32>, ParseIntError> {
    ss.iter().map(|s| u32::from_str_radix(s, 16)).collect()
}
