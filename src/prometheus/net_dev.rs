use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::num::ParseIntError;
use std::path::Path;
use thiserror::Error;

#[derive(Debug)]
pub struct NetDevLine {
    pub name: String,
    pub rx_bytes: u64,
    pub rx_packets: u64,
    pub rx_errors: u64,
    pub rx_dropped: u64,
    pub rx_fifo: u64,
    pub rx_frame: u64,
    pub rx_compressed: u64,
    pub rx_multicast: u64,
    pub tx_bytes: u64,
    pub tx_packets: u64,
    pub tx_errors: u64,
    pub tx_dropped: u64,
    pub tx_fifo: u64,
    pub tx_collisions: u64,
    pub tx_carrier: u64,
    pub tx_compressed: u64,
}

pub type NetDev = HashMap<String, NetDevLine>;

#[derive(Debug, Error)]
pub enum NetDevError {
    #[error("file read error")]
    FileReadError(#[from] io::Error),
    #[error("parse error")]
    ParseError(#[from] ParseIntError),
    #[error("invalid net/dev line, missing colon")]
    MissingColon,
    #[error("invalid net/dev line, empty interface name")]
    EmptyInterfaceName,
}

pub fn net_dev<P: AsRef<Path>>(path: P) -> Result<NetDev, NetDevError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    parse_net_dev(reader)
}

fn parse_net_dev<R: BufRead>(reader: R) -> Result<NetDev, NetDevError> {
    let mut net_dev = NetDev::new();
    let mut lines = reader.lines();

    // Skip the 2 header lines
    lines.next();
    lines.next();

    for line in lines {
        let line = line?;
        let net_dev_line = parse_line(&line)?;
        net_dev.insert(net_dev_line.name.clone(), net_dev_line);
    }

    Ok(net_dev)
}

fn parse_line(raw_line: &str) -> Result<NetDevLine, NetDevError> {
    let idx = raw_line.find(':').ok_or(NetDevError::MissingColon)?;
    let fields: Vec<&str> = raw_line[idx + 1..].split_whitespace().collect();

    let name = raw_line[..idx].trim().to_string();
    if name.is_empty() {
        return Err(NetDevError::EmptyInterfaceName);
    }

    let parse_field = |s: &str| -> Result<u64, NetDevError> {
        Ok(s.parse::<u64>()?)
    };

    Ok(NetDevLine {
        name,
        rx_bytes: parse_field(fields[0])?,
        rx_packets: parse_field(fields[1])?,
        rx_errors: parse_field(fields[2])?,
        rx_dropped: parse_field(fields[3])?,
        rx_fifo: parse_field(fields[4])?,
        rx_frame: parse_field(fields[5])?,
        rx_compressed: parse_field(fields[6])?,
        rx_multicast: parse_field(fields[7])?,
        tx_bytes: parse_field(fields[8])?,
        tx_packets: parse_field(fields[9])?,
        tx_errors: parse_field(fields[10])?,
        tx_dropped: parse_field(fields[11])?,
        tx_fifo: parse_field(fields[12])?,
        tx_collisions: parse_field(fields[13])?,
        tx_carrier: parse_field(fields[14])?,
        tx_compressed: parse_field(fields[15])?,
    })
}

impl NetDev {
    pub fn total(&self) -> NetDevLine {
        let mut total = NetDevLine {
            name: String::new(),
            rx_bytes: 0,
            rx_packets: 0,
            rx_errors: 0,
            rx_dropped: 0,
            rx_fifo: 0,
            rx_frame: 0,
            rx_compressed: 0,
            rx_multicast: 0,
            tx_bytes: 0,
            tx_packets: 0,
            tx_errors: 0,
            tx_dropped: 0,
            tx_fifo: 0,
            tx_collisions: 0,
            tx_carrier: 0,
            tx_compressed: 0,
        };

        let mut names: Vec<String> = Vec::new();
        for line in self.values() {
            names.push(line.name.clone());
            total.rx_bytes += line.rx_bytes;
            total.rx_packets += line.rx_packets;
            total.rx_errors += line.rx_errors;
            total.rx_dropped += line.rx_dropped;
            total.rx_fifo += line.rx_fifo;
            total.rx_frame += line.rx_frame;
            total.rx_compressed += line.rx_compressed;
            total.rx_multicast += line.rx_multicast;
            total.tx_bytes += line.tx_bytes;
            total.tx_packets += line.tx_packets;
            total.tx_errors += line.tx_errors;
            total.tx_dropped += line.tx_dropped;
            total.tx_fifo += line.tx_fifo;
            total.tx_collisions += line.tx_collisions;
            total.tx_carrier += line.tx_carrier;
            total.tx_compressed += line.tx_compressed;
        }

        names.sort();
        total.name = names.join(", ");

        total
    }
}