// Copyright 2018 The Prometheus Authors
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

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

const NETCLASS_PATH: &str = "class/net";

#[derive(Debug, Default)]
pub struct NetClassIface {
    name: String,
    addr_assign_type: Option<i64>,
    addr_len: Option<i64>,
    address: Option<String>,
    broadcast: Option<String>,
    carrier: Option<i64>,
    carrier_changes: Option<i64>,
    carrier_up_count: Option<i64>,
    carrier_down_count: Option<i64>,
    dev_id: Option<i64>,
    dormant: Option<i64>,
    duplex: Option<String>,
    flags: Option<i64>,
    ifalias: Option<String>,
    ifindex: Option<i64>,
    iflink: Option<i64>,
    link_mode: Option<i64>,
    mtu: Option<i64>,
    name_assign_type: Option<i64>,
    netdev_group: Option<i64>,
    operstate: Option<String>,
    phys_port_id: Option<String>,
    phys_port_name: Option<String>,
    phys_switch_id: Option<String>,
    speed: Option<i64>,
    tx_queue_len: Option<i64>,
    type_: Option<i64>,
}

pub type NetClass = HashMap<String, NetClassIface>;

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn net_class_devices(&self) -> Result<Vec<String>, io::Error> {
        let path = Path::new(&self.sys_path).join(NETCLASS_PATH);
        let devices = fs::read_dir(path)?;

        let mut res = Vec::new();
        for device in devices {
            let device = device?;
            if device.file_type()?.is_dir() {
                res.push(device.file_name().into_string().unwrap());
            }
        }

        Ok(res)
    }

    pub fn net_class_by_iface(&self, device_path: &str) -> Result<NetClassIface, io::Error> {
        let path = Path::new(&self.sys_path).join(NETCLASS_PATH).join(device_path);
        let mut iface = parse_net_class_iface(&path)?;
        iface.name = device_path.to_string();
        Ok(iface)
    }

    pub fn net_class(&self) -> Result<NetClass, io::Error> {
        let devices = self.net_class_devices()?;
        let mut net_class = NetClass::new();

        for device in devices {
            let iface = self.net_class_by_iface(&device)?;
            net_class.insert(device, iface);
        }

        Ok(net_class)
    }
}

fn can_ignore_error(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied | io::ErrorKind::InvalidInput
    ) || err.to_string() == "operation not supported"
}

fn parse_net_class_attribute(device_path: &Path, attr_name: &str, iface: &mut NetClassIface) -> Result<(), io::Error> {
    let attr_path = device_path.join(attr_name);
    let value = match fs::read_to_string(&attr_path) {
        Ok(val) => val.trim().to_string(),
        Err(err) if can_ignore_error(&err) => return Ok(()),
        Err(err) => return Err(err),
    };

    match attr_name {
        "addr_assign_type" => iface.addr_assign_type = value.parse().ok(),
        "addr_len" => iface.addr_len = value.parse().ok(),
        "address" => iface.address = Some(value),
        "broadcast" => iface.broadcast = Some(value),
        "carrier" => iface.carrier = value.parse().ok(),
        "carrier_changes" => iface.carrier_changes = value.parse().ok(),
        "carrier_up_count" => iface.carrier_up_count = value.parse().ok(),
        "carrier_down_count" => iface.carrier_down_count = value.parse().ok(),
        "dev_id" => iface.dev_id = value.parse().ok(),
        "dormant" => iface.dormant = value.parse().ok(),
        "duplex" => iface.duplex = Some(value),
        "flags" => iface.flags = value.parse().ok(),
        "ifalias" => iface.ifalias = Some(value),
        "ifindex" => iface.ifindex = value.parse().ok(),
        "iflink" => iface.iflink = value.parse().ok(),
        "link_mode" => iface.link_mode = value.parse().ok(),
        "mtu" => iface.mtu = value.parse().ok(),
        "name_assign_type" => iface.name_assign_type = value.parse().ok(),
        "netdev_group" => iface.netdev_group = value.parse().ok(),
        "operstate" => iface.operstate = Some(value),
        "phys_port_id" => iface.phys_port_id = Some(value),
        "phys_port_name" => iface.phys_port_name = Some(value),
        "phys_switch_id" => iface.phys_switch_id = Some(value),
        "speed" => iface.speed = value.parse().ok(),
        "tx_queue_len" => iface.tx_queue_len = value.parse().ok(),
        "type" => iface.type_ = value.parse().ok(),
        _ => {}
    }

    Ok(())
}

fn parse_net_class_iface(device_path: &Path) -> Result<NetClassIface, io::Error> {
    let mut iface = NetClassIface::default();
    let files = fs::read_dir(device_path)?;

    for file in files {
        let file = file?;
        if file.file_type()?.is_file() {
            parse_net_class_attribute(device_path, &file.file_name().to_string_lossy(), &mut iface)?;
        }
    }

    Ok(iface)
}