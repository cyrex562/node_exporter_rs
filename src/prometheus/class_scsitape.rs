// Copyright 2021 The Prometheus Authors
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

use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

const SCSI_TAPE_CLASS_PATH: &str = "class/scsi_tape";

#[derive(Debug, Default)]
pub struct SCSITapeCounters {
    write_ns: u64,
    read_byte_cnt: u64,
    io_ns: u64,
    write_cnt: u64,
    resid_cnt: u64,
    read_ns: u64,
    in_flight: u64,
    other_cnt: u64,
    read_cnt: u64,
    write_byte_cnt: u64,
}

#[derive(Debug, Default)]
pub struct SCSITape {
    name: String,
    counters: SCSITapeCounters,
}

pub type SCSITapeClass = HashMap<String, SCSITape>;

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn scsi_tape_class(&self) -> Result<SCSITapeClass, io::Error> {
        let path = Path::new(&self.sys_path).join(SCSI_TAPE_CLASS_PATH);
        let dirs = fs::read_dir(path)?;

        let mut stc = SCSITapeClass::new();
        let valid_device = Regex::new(r"^st[0-9]+$").unwrap();

        for dir in dirs {
            let dir = dir?;
            if !valid_device.is_match(&dir.file_name().to_string_lossy()) {
                continue;
            }
            let tape = self.parse_scsi_tape(&dir.file_name().to_string_lossy())?;
            stc.insert(tape.name.clone(), tape);
        }

        Ok(stc)
    }

    fn parse_scsi_tape(&self, name: &str) -> Result<SCSITape, io::Error> {
        let path = Path::new(&self.sys_path).join(SCSI_TAPE_CLASS_PATH).join(name);
        let counters = parse_scsi_tape_statistics(&path)?;

        Ok(SCSITape {
            name: name.to_string(),
            counters,
        })
    }
}

fn parse_scsi_tape_statistics(tape_path: &Path) -> Result<SCSITapeCounters, io::Error> {
    let path = tape_path.join("stats");
    let files = fs::read_dir(path)?;

    let mut counters = SCSITapeCounters::default();
    for file in files {
        let file = file?;
        let name = file.file_name().to_string_lossy().to_string();
        let value = fs::read_to_string(file.path())?.trim().parse::<u64>().unwrap_or(0);

        match name.as_str() {
            "in_flight" => counters.in_flight = value,
            "io_ns" => counters.io_ns = value,
            "other_cnt" => counters.other_cnt = value,
            "read_byte_cnt" => counters.read_byte_cnt = value,
            "read_cnt" => counters.read_cnt = value,
            "read_ns" => counters.read_ns = value,
            "resid_cnt" => counters.resid_cnt = value,
            "write_byte_cnt" => counters.write_byte_cnt = value,
            "write_cnt" => counters.write_cnt = value,
            "write_ns" => counters.write_ns = value,
            _ => {}
        }
    }

    Ok(counters)
}