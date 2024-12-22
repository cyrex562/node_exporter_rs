use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::num::ParseIntError;
use std::path::Path;
use thiserror::Error;

const NET_UNIX_TYPE_STREAM: u64 = 1;
const NET_UNIX_TYPE_DGRAM: u64 = 2;
const NET_UNIX_TYPE_SEQPACKET: u64 = 5;

const NET_UNIX_FLAG_DEFAULT: u64 = 0;
const NET_UNIX_FLAG_LISTEN: u64 = 1 << 16;

const NET_UNIX_STATE_UNCONNECTED: u64 = 1;
const NET_UNIX_STATE_CONNECTING: u64 = 2;
const NET_UNIX_STATE_CONNECTED: u64 = 3;
const NET_UNIX_STATE_DISCONNECTED: u64 = 4;

#[derive(Debug)]
pub struct NetUNIXLine {
    pub kernel_ptr: String,
    pub ref_count: u64,
    pub protocol: u64,
    pub flags: u64,
    pub typ: u64,
    pub state: u64,
    pub inode: u64,
    pub path: Option<String>,
}

#[derive(Debug)]
pub struct NetUNIX {
    pub rows: Vec<NetUNIXLine>,
}

#[derive(Debug, Error)]
pub enum NetUNIXError {
    #[error("file read error")]
    FileReadError(#[from] io::Error),
    #[error("parse error")]
    ParseError(#[from] ParseIntError),
    #[error("invalid format: {0}")]
    InvalidFormat(String),
}

pub fn net_unix<P: AsRef<Path>>(path: P) -> Result<NetUNIX, NetUNIXError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    parse_net_unix(reader)
}

fn parse_net_unix<R: Read>(reader: R) -> Result<NetUNIX, NetUNIXError> {
    let mut rows = Vec::new();
    let mut lines = BufReader::new(reader).lines();

    // Skip the header line
    let header = lines
        .next()
        .ok_or_else(|| NetUNIXError::InvalidFormat("Missing header".to_string()))??;
    let has_inode = header.contains("Inode");

    for line in lines {
        let line = line?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        let row = parse_line(&fields, has_inode)?;
        rows.push(row);
    }

    Ok(NetUNIX { rows })
}

fn parse_line(fields: &[&str], has_inode: bool) -> Result<NetUNIXLine, NetUNIXError> {
    let min_fields = if has_inode { 7 } else { 6 };
    if fields.len() < min_fields {
        return Err(NetUNIXError::InvalidFormat(format!(
            "Expected at least {} fields but got {}",
            min_fields,
            fields.len()
        )));
    }

    let kernel_ptr = fields[0].trim_end_matches(':').to_string();
    let ref_count = u64::from_str_radix(fields[1], 16)?;
    let protocol = u64::from_str_radix(fields[2], 16)?;
    let flags = u64::from_str_radix(fields[3], 16)?;
    let typ = u64::from_str_radix(fields[4], 16)?;
    let state = u64::from_str_radix(fields[5], 16)?;

    let inode = if has_inode {
        u64::from_str(fields[6])?
    } else {
        0
    };

    let path = if fields.len() > min_fields {
        Some(fields[min_fields].to_string())
    } else {
        None
    };

    Ok(NetUNIXLine {
        kernel_ptr,
        ref_count,
        protocol,
        flags,
        typ,
        state,
        inode,
        path,
    })
}

impl NetUNIXLine {
    pub fn parse_users(s: &str) -> Result<u64, ParseIntError> {
        u64::from_str_radix(s, 16)
    }

    pub fn parse_type(s: &str) -> Result<u64, ParseIntError> {
        u64::from_str_radix(s, 16)
    }

    pub fn parse_flags(s: &str) -> Result<u64, ParseIntError> {
        u64::from_str_radix(s, 16)
    }

    pub fn parse_state(s: &str) -> Result<u64, ParseIntError> {
        u64::from_str_radix(s, 16)
    }

    pub fn parse_inode(s: &str) -> Result<u64, ParseIntError> {
        u64::from_str(s)
    }
}

impl ToString for u64 {
    fn to_string(&self) -> String {
        match *self {
            NET_UNIX_TYPE_STREAM => "stream".to_string(),
            NET_UNIX_TYPE_DGRAM => "dgram".to_string(),
            NET_UNIX_TYPE_SEQPACKET => "seqpacket".to_string(),
            NET_UNIX_FLAG_LISTEN => "listen".to_string(),
            NET_UNIX_FLAG_DEFAULT => "default".to_string(),
            NET_UNIX_STATE_UNCONNECTED => "unconnected".to_string(),
            NET_UNIX_STATE_CONNECTING => "connecting".to_string(),
            NET_UNIX_STATE_CONNECTED => "connected".to_string(),
            NET_UNIX_STATE_DISCONNECTED => "disconnected".to_string(),
            _ => "unknown".to_string(),
        }
    }
}
