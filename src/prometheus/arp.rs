// Copyright 2019 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fs;
use std::net::{IpAddr, AddrParseError};
use std::num::ParseIntError;
use std::str::FromStr;
use thiserror::Error;

const ATF_COMPLETE: u8 = 0x02;
const ATF_PERMANENT: u8 = 0x04;
const ATF_PUBLISH: u8 = 0x08;
const ATF_USETRAILERS: u8 = 0x10;
const ATF_NETMASK: u8 = 0x20;
const ATF_DONTPUBLISH: u8 = 0x40;

#[derive(Debug)]
pub struct ARPEntry {
    ip_addr: IpAddr,
    hw_addr: String,
    device: String,
    flags: u8,
}

#[derive(Debug, Error)]
pub enum ARPError {
    #[error("error reading arp file: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("error parsing arp entry: {0}")]
    ParseError(String),
    #[error("invalid IP address: {0}")]
    InvalidIp(#[from] AddrParseError),
    #[error("invalid MAC address: {0}")]
    InvalidMac(#[from] ParseIntError),
}

pub fn gather_arp_entries(proc_path: &str) -> Result<Vec<ARPEntry>, ARPError> {
    let data = fs::read_to_string(format!("{}/net/arp", proc_path))?;
    parse_arp_entries(&data)
}

fn parse_arp_entries(data: &str) -> Result<Vec<ARPEntry>, ARPError> {
    let lines: Vec<&str> = data.lines().collect();
    let mut entries = Vec::new();

    for line in lines.iter().skip(1) {
        let columns: Vec<&str> = line.split_whitespace().collect();
        if columns.len() == 6 {
            let entry = parse_arp_entry(&columns)?;
            entries.push(entry);
        } else if columns.len() != 0 {
            return Err(ARPError::ParseError(format!(
                "unexpected number of columns: {}",
                columns.len()
            )));
        }
    }

    Ok(entries)
}

fn parse_arp_entry(columns: &[&str]) -> Result<ARPEntry, ARPError> {
    let ip_addr = IpAddr::from_str(columns[0])?;
    let hw_addr = columns[3].to_string();
    let device = columns[5].to_string();
    let flags = u8::from_str_radix(columns[2], 16)?;

    Ok(ARPEntry {
        ip_addr,
        hw_addr,
        device,
        flags,
    })
}

impl ARPEntry {
    pub fn is_complete(&self) -> bool {
        self.flags & ATF_COMPLETE != 0
    }
}