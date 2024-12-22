use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use thiserror::Error;

#[derive(Debug)]
pub struct NetSockstat {
    pub used: Option<i32>,
    pub protocols: Vec<NetSockstatProtocol>,
}

#[derive(Debug)]
pub struct NetSockstatProtocol {
    pub protocol: String,
    pub in_use: i32,
    pub orphan: Option<i32>,
    pub tw: Option<i32>,
    pub alloc: Option<i32>,
    pub mem: Option<i32>,
    pub memory: Option<i32>,
}

#[derive(Debug, Error)]
pub enum NetSockstatError {
    #[error("file read error")]
    FileReadError(#[from] io::Error),
    #[error("parse error")]
    ParseError(#[from] std::num::ParseIntError),
    #[error("malformed sockstat line: {0}")]
    MalformedSockstatLine(String),
    #[error("odd number of fields in key/value pairs: {0:?}")]
    OddNumberOfFields(Vec<String>),
}

pub fn net_sockstat<P: AsRef<Path>>(path: P) -> Result<NetSockstat, NetSockstatError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    parse_sockstat(reader)
}

fn parse_sockstat<R: Read>(reader: R) -> Result<NetSockstat, NetSockstatError> {
    let mut stat = NetSockstat {
        used: None,
        protocols: Vec::new(),
    };
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next() {
        let line = line?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 3 {
            return Err(NetSockstatError::MalformedSockstatLine(line));
        }

        let kvs = parse_sockstat_kvs(&fields[1..])?;
        let proto = fields[0].trim_end_matches(':');
        match proto {
            "sockets" => {
                stat.used = Some(kvs["used"]);
            }
            _ => {
                let mut nsp = parse_sockstat_protocol(kvs);
                nsp.protocol = proto.to_string();
                stat.protocols.push(nsp);
            }
        }
    }

    Ok(stat)
}

fn parse_sockstat_kvs(kvs: &[&str]) -> Result<HashMap<String, i32>, NetSockstatError> {
    if kvs.len() % 2 != 0 {
        return Err(NetSockstatError::OddNumberOfFields(
            kvs.iter().map(|&s| s.to_string()).collect(),
        ));
    }

    let mut out = HashMap::new();
    for i in (0..kvs.len()).step_by(2) {
        let key = kvs[i].to_string();
        let value = kvs[i + 1].parse::<i32>()?;
        out.insert(key, value);
    }

    Ok(out)
}

fn parse_sockstat_protocol(kvs: HashMap<String, i32>) -> NetSockstatProtocol {
    let mut nsp = NetSockstatProtocol {
        protocol: String::new(),
        in_use: 0,
        orphan: None,
        tw: None,
        alloc: None,
        mem: None,
        memory: None,
    };

    for (k, v) in kvs {
        match k.as_str() {
            "inuse" => nsp.in_use = v,
            "orphan" => nsp.orphan = Some(v),
            "tw" => nsp.tw = Some(v),
            "alloc" => nsp.alloc = Some(v),
            "mem" => nsp.mem = Some(v),
            "memory" => nsp.memory = Some(v),
            _ => {}
        }
    }

    nsp
}
