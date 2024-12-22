use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Debug)]
pub struct NetProtocolStatLine {
    pub name: String,
    pub size: u64,
    pub sockets: i64,
    pub memory: i64,
    pub pressure: i32,
    pub max_header: u64,
    pub slab: bool,
    pub module_name: String,
    pub capabilities: NetProtocolCapabilities,
}

#[derive(Debug, Default)]
pub struct NetProtocolCapabilities {
    pub close: bool,
    pub connect: bool,
    pub disconnect: bool,
    pub accept: bool,
    pub ioctl: bool,
    pub init: bool,
    pub destroy: bool,
    pub shutdown: bool,
    pub set_sock_opt: bool,
    pub get_sock_opt: bool,
    pub send_msg: bool,
    pub recv_msg: bool,
    pub send_page: bool,
    pub bind: bool,
    pub backlog_rcv: bool,
    pub hash: bool,
    pub unhash: bool,
    pub get_port: bool,
    pub enter_memory_pressure: bool,
}

pub type NetProtocolStats = HashMap<String, NetProtocolStatLine>;

#[derive(Debug, Error)]
pub enum NetProtocolError {
    #[error("file read error")]
    FileReadError(#[from] io::Error),
    #[error("parse error")]
    ParseError(#[from] ParseIntError),
    #[error("invalid format: {0}")]
    InvalidFormat(String),
}

pub fn net_protocols(file_path: &str) -> Result<NetProtocolStats, NetProtocolError> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    parse_net_protocols(reader)
}

fn parse_net_protocols<R: BufRead>(reader: R) -> Result<NetProtocolStats, NetProtocolError> {
    let mut stats = NetProtocolStats::new();
    let mut lines = reader.lines();

    // Skip the header line
    lines.next();

    for line in lines {
        let line = line?;
        let stat_line = parse_line(&line)?;
        stats.insert(stat_line.name.clone(), stat_line);
    }

    Ok(stats)
}

fn parse_line(raw_line: &str) -> Result<NetProtocolStatLine, NetProtocolError> {
    let fields: Vec<&str> = raw_line.split_whitespace().collect();
    if fields.len() < 27 {
        return Err(NetProtocolError::InvalidFormat(raw_line.to_string()));
    }

    let name = fields[0].to_string();
    let size = fields[1].parse::<u64>()?;
    let sockets = fields[2].parse::<i64>()?;
    let memory = fields[3].parse::<i64>()?;
    let pressure = match fields[4] {
        "yes" => 1,
        "no" => 0,
        _ => -1,
    };
    let max_header = fields[5].parse::<u64>()?;
    let slab = match fields[6] {
        "yes" => true,
        "no" => false,
        _ => return Err(NetProtocolError::InvalidFormat(fields[6].to_string())),
    };
    let module_name = fields[7].to_string();

    let capabilities = parse_capabilities(&fields[8..])?;

    Ok(NetProtocolStatLine {
        name,
        size,
        sockets,
        memory,
        pressure,
        max_header,
        slab,
        module_name,
        capabilities,
    })
}

fn parse_capabilities(fields: &[&str]) -> Result<NetProtocolCapabilities, NetProtocolError> {
    let mut capabilities = NetProtocolCapabilities::default();
    let capability_fields = [
        &mut capabilities.close,
        &mut capabilities.connect,
        &mut capabilities.disconnect,
        &mut capabilities.accept,
        &mut capabilities.ioctl,
        &mut capabilities.init,
        &mut capabilities.destroy,
        &mut capabilities.shutdown,
        &mut capabilities.set_sock_opt,
        &mut capabilities.get_sock_opt,
        &mut capabilities.send_msg,
        &mut capabilities.recv_msg,
        &mut capabilities.send_page,
        &mut capabilities.bind,
        &mut capabilities.backlog_rcv,
        &mut capabilities.hash,
        &mut capabilities.unhash,
        &mut capabilities.get_port,
        &mut capabilities.enter_memory_pressure,
    ];

    for (i, &field) in fields.iter().enumerate() {
        match field {
            "y" => *capability_fields[i] = true,
            "n" => *capability_fields[i] = false,
            _ => return Err(NetProtocolError::InvalidFormat(field.to_string())),
        }
    }

    Ok(capabilities)
}
