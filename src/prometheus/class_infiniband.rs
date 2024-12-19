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
use std::io;
use std::path::Path;
use std::collections::HashMap;

const INFINIBAND_CLASS_PATH: &str = "class/infiniband";

#[derive(Debug, Default)]
pub struct InfiniBandCounters {
    legacy_port_multicast_rcv_packets: Option<u64>,
    legacy_port_multicast_xmit_packets: Option<u64>,
    legacy_port_rcv_data_64: Option<u64>,
    legacy_port_rcv_packets_64: Option<u64>,
    legacy_port_unicast_rcv_packets: Option<u64>,
    legacy_port_unicast_xmit_packets: Option<u64>,
    legacy_port_xmit_data_64: Option<u64>,
    legacy_port_xmit_packets_64: Option<u64>,
    excessive_buffer_overrun_errors: Option<u64>,
    link_downed: Option<u64>,
    link_error_recovery: Option<u64>,
    local_link_integrity_errors: Option<u64>,
    multicast_rcv_packets: Option<u64>,
    multicast_xmit_packets: Option<u64>,
    port_rcv_constraint_errors: Option<u64>,
    port_rcv_data: Option<u64>,
    port_rcv_discards: Option<u64>,
    port_rcv_errors: Option<u64>,
    port_rcv_packets: Option<u64>,
    port_rcv_remote_physical_errors: Option<u64>,
    port_rcv_switch_relay_errors: Option<u64>,
    port_xmit_constraint_errors: Option<u64>,
    port_xmit_data: Option<u64>,
    port_xmit_discards: Option<u64>,
    port_xmit_packets: Option<u64>,
    port_xmit_wait: Option<u64>,
    symbol_error: Option<u64>,
    unicast_rcv_packets: Option<u64>,
    unicast_xmit_packets: Option<u64>,
    vl15_dropped: Option<u64>,
}

#[derive(Debug, Default)]
pub struct InfiniBandHwCounters {
    duplicate_request: Option<u64>,
    implied_nak_seq_err: Option<u64>,
    lifespan: Option<u64>,
    local_ack_timeout_err: Option<u64>,
    np_cnp_sent: Option<u64>,
    np_ecn_marked_roce_packets: Option<u64>,
    out_of_buffer: Option<u64>,
    out_of_sequence: Option<u64>,
    packet_seq_err: Option<u64>,
    req_cqe_error: Option<u64>,
    req_cqe_flush_error: Option<u64>,
    req_remote_access_errors: Option<u64>,
    req_remote_invalid_request: Option<u64>,
    resp_cqe_error: Option<u64>,
    resp_cqe_flush_error: Option<u64>,
    resp_local_length_error: Option<u64>,
    resp_remote_access_errors: Option<u64>,
    rnr_nak_retry_err: Option<u64>,
    roce_adp_retrans: Option<u64>,
    roce_adp_retrans_to: Option<u64>,
    roce_slow_restart: Option<u64>,
    roce_slow_restart_cnps: Option<u64>,
    roce_slow_restart_trans: Option<u64>,
    rp_cnp_handled: Option<u64>,
    rp_cnp_ignored: Option<u64>,
    rx_atomic_requests: Option<u64>,
    rx_dct_connect: Option<u64>,
    rx_icrc_encapsulated: Option<u64>,
    rx_read_requests: Option<u64>,
    rx_write_requests: Option<u64>,
}

#[derive(Debug, Default)]
pub struct InfiniBandPort {
    name: String,
    port: u32,
    state: String,
    state_id: u32,
    phys_state: String,
    phys_state_id: u32,
    rate: u64,
    counters: InfiniBandCounters,
    hw_counters: InfiniBandHwCounters,
}

#[derive(Debug, Default)]
pub struct InfiniBandDevice {
    name: String,
    board_id: String,
    firmware_version: String,
    hca_type: String,
    ports: HashMap<u32, InfiniBandPort>,
}

pub type InfiniBandClass = HashMap<String, InfiniBandDevice>;

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn infiniband_class(&self) -> Result<InfiniBandClass, io::Error> {
        let path = Path::new(&self.sys_path).join(INFINIBAND_CLASS_PATH);
        let dirs = fs::read_dir(path)?;

        let mut ibc = InfiniBandClass::new();
        for dir in dirs {
            let dir = dir?;
            let device = self.parse_infiniband_device(&dir.file_name().to_string_lossy())?;
            ibc.insert(device.name.clone(), device);
        }

        Ok(ibc)
    }

    fn parse_infiniband_device(&self, name: &str) -> Result<InfiniBandDevice, io::Error> {
        let path = Path::new(&self.sys_path).join(INFINIBAND_CLASS_PATH).join(name);
        let mut device = InfiniBandDevice {
            name: name.to_string(),
            ..Default::default()
        };

        device.firmware_version = read_file_to_string(&path.join("fw_ver"))?;
        device.board_id = read_file_to_string(&path.join("board_id")).unwrap_or_default();
        device.hca_type = read_file_to_string(&path.join("hca_type")).unwrap_or_default();

        let ports_path = path.join("ports");
        let ports = fs::read_dir(ports_path)?;

        for port in ports {
            let port = port?;
            let port_number = port.file_name().to_string_lossy().parse::<u32>().unwrap();
            let ib_port = self.parse_infiniband_port(name, port_number)?;
            device.ports.insert(port_number, ib_port);
        }

        Ok(device)
    }

    fn parse_infiniband_port(&self, name: &str, port: u32) -> Result<InfiniBandPort, io::Error> {
        let port_path = Path::new(&self.sys_path).join(INFINIBAND_CLASS_PATH).join(name).join("ports").join(port.to_string());
        let mut ib_port = InfiniBandPort {
            name: name.to_string(),
            port,
            ..Default::default()
        };

        let state_content = read_file_to_string(&port_path.join("state"))?;
        let (state_id, state_name) = parse_state(&state_content)?;
        ib_port.state = state_name;
        ib_port.state_id = state_id;

        let phys_state_content = read_file_to_string(&port_path.join("phys_state"))?;
        let (phys_state_id, phys_state_name) = parse_state(&phys_state_content)?;
        ib_port.phys_state = phys_state_name;
        ib_port.phys_state_id = phys_state_id;

        let rate_content = read_file_to_string(&port_path.join("rate"))?;
        ib_port.rate = parse_rate(&rate_content)?;

        if !name.starts_with("irdma") {
            ib_port.counters = parse_infiniband_counters(&port_path)?;
        }

        if name.starts_with("irdma") || name.starts_with("mlx5_") {
            ib_port.hw_counters = parse_infiniband_hw_counters(&port_path)?;
        }

        Ok(ib_port)
    }
}

fn read_file_to_string(path: &Path) -> Result<String, io::Error> {
    fs::read_to_string(path).map(|s| s.trim().to_string())
}

fn parse_state(s: &str) -> Result<(u32, String), io::Error> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("failed to split {} into 'ID: NAME'", s)));
    }
    let id = parts[0].trim().parse::<u32>()?;
    let name = parts[1].trim().to_string();
    Ok((id, name))
}

fn parse_rate(s: &str) -> Result<u64, io::Error> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("failed to split {}", s)));
    }
    let value = parts[0].trim().parse::<f64>()?;
    Ok((value * 125_000_000.0) as u64)
}

fn parse_infiniband_counters(port_path: &Path) -> Result<InfiniBandCounters, io::Error> {
    let path = port_path.join("counters");
    let files = fs::read_dir(path)?;

    let mut counters = InfiniBandCounters::default();
    for file in files {
        let file = file?;
        if !file.file_type()?.is_file() {
            continue;
        }

        let name = file.file_name().to_string_lossy().to_string();
        let value = read_file_to_string(&file.path())?.parse().ok();

        match name.as_str() {
            "excessive_buffer_overrun_errors" => counters.excessive_buffer_overrun_errors = value,
            "link_downed" => counters.link_downed = value,
            "link_error_recovery" => counters.link_error_recovery = value,
            "local_link_integrity_errors" => counters.local_link_integrity_errors = value,
            "multicast_rcv_packets" => counters.multicast_rcv_packets = value,
            "multicast_xmit_packets" => counters.multicast_xmit_packets = value,
            "port_rcv_constraint_errors" => counters.port_rcv_constraint_errors = value,
            "port_rcv_data" => counters.port_rcv_data = value.map(|v| v * 4),
            "port_rcv_discards" => counters.port_rcv_discards = value,
            "port_rcv_errors" => counters.port_rcv_errors = value,
            "port_rcv_packets" => counters.port_rcv_packets = value,
            "port_rcv_remote_physical_errors" => counters.port_rcv_remote_physical_errors = value,
            "port_rcv_switch_relay_errors" => counters.port_rcv_switch_relay_errors = value,
            "port_xmit_constraint_errors" => counters.port_xmit_constraint_errors = value,
            "port_xmit_data" => counters.port_xmit_data = value.map(|v| v * 4),
            "port_xmit_discards" => counters.port_xmit_discards = value,
            "port_xmit_packets" => counters.port_xmit_packets = value,
            "port_xmit_wait" => counters.port_xmit_wait = value,
            "symbol_error" => counters.symbol_error = value,
            "unicast_rcv_packets" => counters.unicast_rcv_packets = value,
            "unicast_xmit_packets" => counters.unicast_xmit_packets = value,
            "VL15_dropped" => counters.vl15_dropped = value,
            _ => {}
        }
    }

    let path = port_path.join("counters_ext");
    let files = fs::read_dir(path)?;

    for file in files {
        let file = file?;
        if !file.file_type()?.is_file() {
            continue;
        }

        let name = file.file_name().to_string_lossy().to_string();
        let value = read_file_to_string(&file.path())?.parse().ok();

        match name.as_str() {
            "port_multicast_rcv_packets" => counters.legacy_port_multicast_rcv_packets = value,
            "port_multicast_xmit_packets" => counters.legacy_port_multicast_xmit_packets = value,
            "port_rcv_data_64" => counters.legacy_port_rcv_data_64 = value.map(|v| v * 4),
            "port_rcv_packets_64" => counters.legacy_port_rcv_packets_64 = value,
            "port_unicast_rcv_packets" => counters.legacy_port_unicast_rcv_packets = value,
            "port_unicast_xmit_packets" => counters.legacy_port_unicast_xmit_packets = value,
            "port_xmit_data_64" => counters.legacy_port_xmit_data_64 = value.map(|v| v * 4),
            "port_xmit_packets_64" => counters.legacy_port_xmit_packets_64 = value,
            _ => {}
        }
    }

    Ok(counters)
}

fn parse_infiniband_hw_counters(port_path: &Path) -> Result<InfiniBandHwCounters, io::Error> {
    let path = port_path.join("hw_counters");
    let files = fs::read_dir(path)?;

    let mut hw_counters = InfiniBandHwCounters::default();
    for file in files {
        let file = file?;
        if !file.file_type()?.is_file() {
            continue;
        }

        let name = file.file_name().to_string_lossy().to_string();
        let value = read_file_to_string(&file.path())?.parse().ok();

        match name.as_str() {
            "duplicate_request" => hw_counters.duplicate_request = value,
            "implied_nak_seq_err" => hw_counters.implied_nak_seq_err = value,
            "lifespan" => hw_counters.lifespan = value,
            "local_ack_timeout_err" => hw_counters.local_ack_timeout_err = value,
            "np_cnp_sent" => hw_counters.np_cnp_sent = value,
            "np_ecn_marked_roce_packets" => hw_counters.np_ecn_marked_roce_packets = value,
            "out_of_buffer" => hw_counters.out_of_buffer = value,
            "out_of_sequence" => hw_counters.out_of_sequence = value,
            "packet_seq_err" => hw_counters.packet_seq_err = value,
            "req_cqe_error" => hw_counters.req_cqe_error = value,
            "req_cqe_flush_error" => hw_counters.req_cqe_flush_error = value,
            "req_remote_access_errors" => hw_counters.req_remote_access_errors = value,
            "req_remote_invalid_request" => hw_counters.req_remote_invalid_request = value,
            "resp_cqe_error" => hw_counters.resp_cqe_error = value,
            "resp_cqe_flush_error" => hw_counters.resp_cqe_flush_error = value,
            "resp_local_length_error" => hw_counters.resp_local_length_error = value,
            "resp_remote_access_errors" => hw_counters.resp_remote_access_errors = value,
            "rnr_nak_retry_err" => hw_counters.rnr_nak_retry_err = value,
            "roce_adp_retrans" => hw_counters.roce_adp_retrans = value,
            "roce_adp_retrans_to" => hw_counters.roce_adp_retrans_to = value,
            "roce_slow_restart" => hw_counters.roce_slow_restart = value,
            "roce_slow_restart_cnps" => hw_counters.roce_slow_restart_cnps = value,
            "roce_slow_restart_trans" => hw_counters.roce_slow_restart_trans = value,
            "rp_cnp_handled" => hw_counters.rp_cnp_handled = value,
            "rp_cnp_ignored" => hw_counters.rp_cnp_ignored = value,
            "rx_atomic_requests" => hw_counters.rx_atomic_requests = value,
            "rx_dct_connect" => hw_counters.rx_dct_connect = value,
            "rx_icrc_encapsulated" => hw_counters.rx_icrc_encapsulated = value,
            "rx_read_requests" => hw_counters.rx_read_requests = value,
            "rx_write_requests" => hw_counters.rx_write_requests = value,
            _ => {}
        }
    }

    Ok(hw_counters)
}