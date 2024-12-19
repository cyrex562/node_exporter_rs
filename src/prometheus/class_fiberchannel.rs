// Copyright 2020 The Prometheus Authors
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
use std::io;
use std::path::Path;
use std::collections::HashMap;

const FIBRECHANNEL_CLASS_PATH: &str = "class/fc_host";

#[derive(Debug, Default)]
pub struct FibreChannelCounters {
    dumped_frames: Option<u64>,
    error_frames: Option<u64>,
    invalid_crc_count: Option<u64>,
    rx_frames: Option<u64>,
    rx_words: Option<u64>,
    tx_frames: Option<u64>,
    tx_words: Option<u64>,
    seconds_since_last_reset: Option<u64>,
    invalid_tx_word_count: Option<u64>,
    link_failure_count: Option<u64>,
    loss_of_sync_count: Option<u64>,
    loss_of_signal_count: Option<u64>,
    nos_count: Option<u64>,
    fcp_packet_aborts: Option<u64>,
}

#[derive(Debug, Default)]
pub struct FibreChannelHost {
    name: Option<String>,
    speed: Option<String>,
    port_state: Option<String>,
    port_type: Option<String>,
    symbolic_name: Option<String>,
    node_name: Option<String>,
    port_id: Option<String>,
    port_name: Option<String>,
    fabric_name: Option<String>,
    dev_loss_tmo: Option<String>,
    supported_classes: Option<String>,
    supported_speeds: Option<String>,
    counters: Option<FibreChannelCounters>,
}

pub type FibreChannelClass = HashMap<String, FibreChannelHost>;

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn fibre_channel_class(&self) -> Result<FibreChannelClass, io::Error> {
        let path = Path::new(&self.sys_path).join(FIBRECHANNEL_CLASS_PATH);
        let dirs = fs::read_dir(path)?;

        let mut fcc = FibreChannelClass::new();
        for dir in dirs {
            let dir = dir?;
            let host = self.parse_fibre_channel_host(&dir.file_name().to_string_lossy())?;
            fcc.insert(host.name.clone().unwrap(), host);
        }

        Ok(fcc)
    }

    fn parse_fibre_channel_host(&self, name: &str) -> Result<FibreChannelHost, io::Error> {
        let path = Path::new(&self.sys_path).join(FIBRECHANNEL_CLASS_PATH).join(name);
        let mut host = FibreChannelHost { name: Some(name.to_string()), ..Default::default() };

        for field in &["speed", "port_state", "port_type", "node_name", "port_id", "port_name", "fabric_name", "dev_loss_tmo", "symbolic_name", "supported_classes", "supported_speeds"] {
            let value = read_file_to_string(&path.join(field))?;
            match *field {
                "speed" => host.speed = Some(value),
                "port_state" => host.port_state = Some(value),
                "port_type" => host.port_type = Some(value),
                "node_name" => host.node_name = Some(trim_hex_prefix(&value)),
                "port_id" => host.port_id = Some(trim_hex_prefix(&value)),
                "port_name" => host.port_name = Some(trim_hex_prefix(&value)),
                "fabric_name" => host.fabric_name = Some(trim_hex_prefix(&value)),
                "dev_loss_tmo" => host.dev_loss_tmo = Some(value),
                "supported_classes" => host.supported_classes = Some(value),
                "supported_speeds" => host.supported_speeds = Some(value),
                "symbolic_name" => host.symbolic_name = Some(value),
                _ => {}
            }
        }

        host.counters = Some(parse_fibre_channel_statistics(&path)?);

        Ok(host)
    }
}

fn read_file_to_string(path: &Path) -> Result<String, io::Error> {
    fs::read_to_string(path).map(|s| s.trim().to_string())
}

fn trim_hex_prefix(value: &str) -> String {
    if value.starts_with("0x") {
        value[2..].to_string()
    } else {
        value.to_string()
    }
}

fn parse_fibre_channel_statistics(host_path: &Path) -> Result<FibreChannelCounters, io::Error> {
    let path = host_path.join("statistics");
    let files = fs::read_dir(path)?;

    let mut counters = FibreChannelCounters::default();
    for file in files {
        let file = file?;
        if !file.file_type()?.is_file() || file.file_name() == "reset_statistics" {
            continue;
        }

        let name = file.file_name().to_string_lossy().to_string();
        let value = read_file_to_string(&file.path())?.parse().ok();

        match name.as_str() {
            "dumped_frames" => counters.dumped_frames = value,
            "error_frames" => counters.error_frames = value,
            "invalid_crc_count" => counters.invalid_crc_count = value,
            "rx_frames" => counters.rx_frames = value,
            "rx_words" => counters.rx_words = value,
            "tx_frames" => counters.tx_frames = value,
            "tx_words" => counters.tx_words = value,
            "seconds_since_last_reset" => counters.seconds_since_last_reset = value,
            "invalid_tx_word_count" => counters.invalid_tx_word_count = value,
            "link_failure_count" => counters.link_failure_count = value,
            "loss_of_sync_count" => counters.loss_of_sync_count = value,
            "loss_of_signal_count" => counters.loss_of_signal_count = value,
            "nos_count" => counters.nos_count = value,
            "fcp_packet_aborts" => counters.fcp_packet_aborts = value,
            _ => {}
        }
    }

    Ok(counters)
}