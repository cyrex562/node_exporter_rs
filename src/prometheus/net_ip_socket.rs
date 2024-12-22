use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::net::IpAddr;
use std::num::ParseIntError;
use std::str::FromStr;
use thiserror::Error;

const READ_LIMIT: u64 = 4294967296; // 4 GiB

#[derive(Debug)]
pub struct NetIPSocket(Vec<NetIPSocketLine>);

#[derive(Debug)]
pub struct NetIPSocketSummary {
    pub tx_queue_length: u64,
    pub rx_queue_length: u64,
    pub used_sockets: u64,
    pub drops: Option<u64>,
}

#[derive(Debug)]
pub struct NetIPSocketLine {
    pub sl: u64,
    pub local_addr: IpAddr,
    pub local_port: u64,
    pub rem_addr: IpAddr,
    pub rem_port: u64,
    pub st: u64,
    pub tx_queue: u64,
    pub rx_queue: u64,
    pub uid: u64,
    pub inode: u64,
    pub drops: Option<u64>,
}

#[derive(Debug, Error)]
pub enum NetIPSocketError {
    #[error("file read error")]
    FileReadError(#[from] io::Error),
    #[error("parse error")]
    ParseError(#[from] ParseIntError),
    #[error("invalid format: {0}")]
    InvalidFormat(String),
}

pub fn new_net_ip_socket(file: &str) -> Result<NetIPSocket, NetIPSocketError> {
    let f = File::open(file)?;
    let reader = BufReader::new(io::BufReader::new(f).take(READ_LIMIT));
    let mut lines = reader.lines();

    // Skip the first line with headers
    lines.next();

    let mut net_ip_socket = Vec::new();
    let is_udp = file.contains("udp");

    for line in lines {
        let line = line?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        let socket_line = parse_net_ip_socket_line(&fields, is_udp)?;
        net_ip_socket.push(socket_line);
    }

    Ok(NetIPSocket(net_ip_socket))
}

pub fn new_net_ip_socket_summary(file: &str) -> Result<NetIPSocketSummary, NetIPSocketError> {
    let f = File::open(file)?;
    let reader = BufReader::new(io::BufReader::new(f).take(READ_LIMIT));
    let mut lines = reader.lines();

    // Skip the first line with headers
    lines.next();

    let mut summary = NetIPSocketSummary {
        tx_queue_length: 0,
        rx_queue_length: 0,
        used_sockets: 0,
        drops: None,
    };
    let mut udp_packet_drops = 0;
    let is_udp = file.contains("udp");

    for line in lines {
        let line = line?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        let socket_line = parse_net_ip_socket_line(&fields, is_udp)?;
        summary.tx_queue_length += socket_line.tx_queue;
        summary.rx_queue_length += socket_line.rx_queue;
        summary.used_sockets += 1;
        if is_udp {
            udp_packet_drops += socket_line.drops.unwrap_or(0);
            summary.drops = Some(udp_packet_drops);
        }
    }

    Ok(summary)
}

fn parse_ip(hex_ip: &str) -> Result<IpAddr, NetIPSocketError> {
    let byte_ip = hex::decode(hex_ip)?;
    match byte_ip.len() {
        4 => Ok(IpAddr::V4(byte_ip.into())),
        16 => Ok(IpAddr::V6(byte_ip.into())),
        _ => Err(NetIPSocketError::InvalidFormat(hex_ip.to_string())),
    }
}

fn parse_net_ip_socket_line(
    fields: &[&str],
    is_udp: bool,
) -> Result<NetIPSocketLine, NetIPSocketError> {
    if fields.len() < 10 {
        return Err(NetIPSocketError::InvalidFormat(fields.join(" ")));
    }

    let sl = fields[0]
        .split(':')
        .next()
        .ok_or_else(|| NetIPSocketError::InvalidFormat(fields[0].to_string()))?;
    let sl = u64::from_str(sl)?;

    let local_addr_port = fields[1].split(':').collect::<Vec<&str>>();
    let local_addr = parse_ip(local_addr_port[0])?;
    let local_port = u64::from_str_radix(local_addr_port[1], 16)?;

    let rem_addr_port = fields[2].split(':').collect::<Vec<&str>>();
    let rem_addr = parse_ip(rem_addr_port[0])?;
    let rem_port = u64::from_str_radix(rem_addr_port[1], 16)?;

    let st = u64::from_str_radix(fields[3], 16)?;

    let tx_rx_queue = fields[4].split(':').collect::<Vec<&str>>();
    let tx_queue = u64::from_str_radix(tx_rx_queue[0], 16)?;
    let rx_queue = u64::from_str_radix(tx_rx_queue[1], 16)?;

    let uid = u64::from_str(fields[7])?;
    let inode = u64::from_str(fields[9])?;

    let drops = if is_udp {
        Some(u64::from_str(fields[12])?)
    } else {
        None
    };

    Ok(NetIPSocketLine {
        sl,
        local_addr,
        local_port,
        rem_addr,
        rem_port,
        st,
        tx_queue,
        rx_queue,
        uid,
        inode,
        drops,
    })
}
