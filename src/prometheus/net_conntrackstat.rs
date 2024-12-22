use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::num::ParseIntError;
use std::path::Path;
use thiserror::Error;

#[derive(Debug)]
pub struct ConntrackStatEntry {
    pub entries: u64,
    pub searched: u64,
    pub found: u64,
    pub new: u64,
    pub invalid: u64,
    pub ignore: u64,
    pub delete: u64,
    pub delete_list: u64,
    pub insert: u64,
    pub insert_failed: u64,
    pub drop: u64,
    pub early_drop: u64,
    pub search_restart: Option<u64>,
}

#[derive(Debug, Error)]
pub enum ConntrackStatError {
    #[error("file read error")]
    FileReadError(#[from] io::Error),
    #[error("parse error")]
    ParseError(#[from] ParseIntError),
    #[error("invalid number of fields: {0}")]
    InvalidNumberOfFields(usize),
}

pub fn conntrack_stat<P: AsRef<Path>>(path: P) -> Result<Vec<ConntrackStatEntry>, ConntrackStatError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    parse_conntrack_stat(reader)
}

fn parse_conntrack_stat<R: Read>(reader: R) -> Result<Vec<ConntrackStatEntry>, ConntrackStatError> {
    let mut entries = Vec::new();
    let mut lines = BufReader::new(reader).lines();

    // Skip the header line
    lines.next();

    for line in lines {
        let line = line?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        let entry = parse_conntrack_stat_entry(&fields)?;
        entries.push(entry);
    }

    Ok(entries)
}

fn parse_conntrack_stat_entry(fields: &[&str]) -> Result<ConntrackStatEntry, ConntrackStatError> {
    let entries: Result<Vec<u64>, _> = fields.iter().map(|&s| u64::from_str_radix(s, 16)).collect();
    let entries = entries?;

    let num_entries = entries.len();
    if num_entries < 16 || num_entries > 17 {
        return Err(ConntrackStatError::InvalidNumberOfFields(num_entries));
    }

    Ok(ConntrackStatEntry {
        entries: entries[0],
        searched: entries[1],
        found: entries[2],
        new: entries[3],
        invalid: entries[4],
        ignore: entries[5],
        delete: entries[6],
        delete_list: entries[7],
        insert: entries[8],
        insert_failed: entries[9],
        drop: entries[10],
        early_drop: entries[11],
        search_restart: if num_entries == 17 { Some(entries[16]) } else { None },
    })
}