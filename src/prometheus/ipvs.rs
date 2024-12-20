use std::fs::File;
use std::io::{self, BufRead, Read};
use std::net::IpAddr;
use std::path::Path;
use std::str::FromStr;
use hex;
use thiserror::Error;

#[derive(Debug)]
pub struct IpvsStats {
    connections: u64,
    incoming_packets: u64,
    outgoing_packets: u64,
    incoming_bytes: u64,
    outgoing_bytes: u64,
}

#[derive(Debug)]
pub struct IpvsBackendStatus {
    local_address: IpAddr,
    remote_address: IpAddr,
    local_port: u16,
    remote_port: u16,
    local_mark: String,
    proto: String,
    active_conn: u64,
    inact_conn: u64,
    weight: u64,
}

#[derive(Debug, Error)]
pub enum ProcFsError {
    #[error("file parse error: {0}")]
    FileParseError(String),
    #[error("io error: {0}")]
    IoError(#[from] io::Error),
    #[error("hex decode error: {0}")]
    HexDecodeError(#[from] hex::FromHexError),
}

pub struct ProcFs {
    proc: String,
}

impl ProcFs {
    pub fn new(proc: &str) -> Self {
        ProcFs { proc: proc.to_string() }
    }

    pub fn ipvs_stats(&self) -> Result<IpvsStats, ProcFsError> {
        let data = std::fs::read_to_string(Path::new(&self.proc).join("net/ip_vs_stats"))?;
        parse_ipvs_stats(&data)
    }

    pub fn ipvs_backend_status(&self) -> Result<Vec<IpvsBackendStatus>, ProcFsError> {
        let file = File::open(Path::new(&self.proc).join("net/ip_vs"))?;
        parse_ipvs_backend_status(file)
    }
}

fn parse_ipvs_stats(data: &str) -> Result<IpvsStats, ProcFsError> {
    let lines: Vec<&str> = data.splitn(4, '\n').collect();
    if lines.len() != 4 {
        return Err(ProcFsError::FileParseError("ip_vs_stats corrupt: too short".to_string()));
    }

    let fields: Vec<&str> = lines[2].split_whitespace().collect();
    if fields.len() != 5 {
        return Err(ProcFsError::FileParseError("ip_vs_stats corrupt: unexpected number of fields".to_string()));
    }

    Ok(IpvsStats {
        connections: u64::from_str_radix(fields[0], 16)?,
        incoming_packets: u64::from_str_radix(fields[1], 16)?,
        outgoing_packets: u64::from_str_radix(fields[2], 16)?,
        incoming_bytes: u64::from_str_radix(fields[3], 16)?,
        outgoing_bytes: u64::from_str_radix(fields[4], 16)?,
    })
}

fn parse_ipvs_backend_status<R: Read>(reader: R) -> Result<Vec<IpvsBackendStatus>, ProcFsError> {
    let mut status = Vec::new();
    let scanner = io::BufReader::new(reader).lines();
    let mut proto = String::new();
    let mut local_mark = String::new();
    let mut local_address = IpAddr::from([0, 0, 0, 0]);
    let mut local_port = 0;

    for line in scanner {
        let line = line?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.is_empty() {
            continue;
        }
        match fields[0] {
            "IP" | "Prot" | "RemoteAddress:Port" => continue,
            "TCP" | "UDP" => {
                if fields.len() < 2 {
                    continue;
                }
                proto = fields[0].to_string();
                local_mark = String::new();
                let (addr, port) = parse_ip_port(fields[1])?;
                local_address = addr;
                local_port = port;
            }
            "FWM" => {
                if fields.len() < 2 {
                    continue;
                }
                proto = fields[0].to_string();
                local_mark = fields[1].to_string();
                local_address = IpAddr::from([0, 0, 0, 0]);
                local_port = 0;
            }
            "->" => {
                if fields.len() < 6 {
                    continue;
                }
                let (remote_address, remote_port) = parse_ip_port(fields[1])?;
                let weight = fields[3].parse()?;
                let active_conn = fields[4].parse()?;
                let inact_conn = fields[5].parse()?;
                status.push(IpvsBackendStatus {
                    local_address,
                    local_port,
                    local_mark: local_mark.clone(),
                    remote_address,
                    remote_port,
                    proto: proto.clone(),
                    weight,
                    active_conn,
                    inact_conn,
                });
            }
            _ => continue,
        }
    }
    Ok(status)
}

fn parse_ip_port(s: &str) -> Result<(IpAddr, u16), ProcFsError> {
    let ip;
    match s.len() {
        13 => {
            ip = IpAddr::from(hex::decode(&s[0..8])?);
        }
        46 => {
            ip = IpAddr::from_str(&s[1..40]).map_err(|_| ProcFsError::FileParseError(format!("Invalid IPv6 addr: {}", &s[1..40])))?;
        }
        _ => return Err(ProcFsError::FileParseError(format!("Unexpected IP:Port: {}", s))),
    }

    let port = u16::from_str_radix(&s[s.len() - 4..], 16)?;
    Ok((ip, port))
}