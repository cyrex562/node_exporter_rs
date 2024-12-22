use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::num::ParseIntError;
use std::path::Path;
use thiserror::Error;

const BLACKHOLE_REPRESENTATION: &str = "*";
const BLACKHOLE_IFACE_NAME: &str = "blackhole";
const ROUTE_LINE_COLUMNS: usize = 11;

#[derive(Debug)]
pub struct NetRouteLine {
    pub iface: String,
    pub destination: u32,
    pub gateway: u32,
    pub flags: u32,
    pub refcnt: u32,
    pub use_: u32,
    pub metric: u32,
    pub mask: u32,
    pub mtu: u32,
    pub window: u32,
    pub irtt: u32,
}

#[derive(Debug, Error)]
pub enum NetRouteError {
    #[error("file read error")]
    FileReadError(#[from] io::Error),
    #[error("parse error")]
    ParseError(#[from] ParseIntError),
    #[error("invalid route line, number of fields: {0}")]
    InvalidRouteLine(usize),
}

pub fn net_route<P: AsRef<Path>>(path: P) -> Result<Vec<NetRouteLine>, NetRouteError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    parse_net_route(reader)
}

fn parse_net_route<R: Read>(reader: R) -> Result<Vec<NetRouteLine>, NetRouteError> {
    let mut routelines = Vec::new();
    let mut lines = BufReader::new(reader).lines();

    // Skip the header line
    lines.next();

    for line in lines {
        let line = line?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        let routeline = parse_net_route_line(&fields)?;
        routelines.push(routeline);
    }

    Ok(routelines)
}

fn parse_net_route_line(fields: &[&str]) -> Result<NetRouteLine, NetRouteError> {
    if fields.len() != ROUTE_LINE_COLUMNS {
        return Err(NetRouteError::InvalidRouteLine(fields.len()));
    }

    let iface = if fields[0] == BLACKHOLE_REPRESENTATION {
        BLACKHOLE_IFACE_NAME.to_string()
    } else {
        fields[0].to_string()
    };

    let destination = u32::from_str_radix(fields[1], 16)?;
    let gateway = u32::from_str_radix(fields[2], 16)?;
    let flags = u32::from_str(fields[3])?;
    let refcnt = u32::from_str(fields[4])?;
    let use_ = u32::from_str(fields[5])?;
    let metric = u32::from_str(fields[6])?;
    let mask = u32::from_str_radix(fields[7], 16)?;
    let mtu = u32::from_str(fields[8])?;
    let window = u32::from_str(fields[9])?;
    let irtt = u32::from_str(fields[10])?;

    Ok(NetRouteLine {
        iface,
        destination,
        gateway,
        flags,
        refcnt,
        use_,
        metric,
        mask,
        mtu,
        window,
        irtt,
    })
}
