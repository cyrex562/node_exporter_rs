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

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::str::FromStr;

const SAS_PHY_CLASS_PATH: &str = "class/sas_phy";

#[derive(Debug)]
pub struct SASPhy {
    name: String,
    sas_address: String,
    sas_port: String,
    device_type: String,
    initiator_port_protocols: Vec<String>,
    invalid_dword_count: i32,
    loss_of_dword_sync_count: i32,
    maximum_linkrate: f64,
    maximum_linkrate_hw: f64,
    minimum_linkrate: f64,
    minimum_linkrate_hw: f64,
    negotiated_linkrate: f64,
    phy_identifier: String,
    phy_reset_problem_count: i32,
    running_disparity_error_count: i32,
    target_port_protocols: Vec<String>,
}

pub type SASPhyClass = HashMap<String, SASPhy>;

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn sas_phy_class(&self) -> Result<SASPhyClass, io::Error> {
        let path = Path::new(&self.sys_path).join(SAS_PHY_CLASS_PATH);
        let dirs = fs::read_dir(path)?;

        let mut spc = SASPhyClass::new();
        for dir in dirs {
            let dir = dir?;
            let phy = self.parse_sas_phy(&dir.file_name().to_string_lossy())?;
            spc.insert(phy.name.clone(), phy);
        }

        Ok(spc)
    }

    fn parse_sas_phy(&self, name: &str) -> Result<SASPhy, io::Error> {
        let phypath = Path::new(&self.sys_path).join(SAS_PHY_CLASS_PATH).join(name);
        let phydevicepath = phypath.join("device");

        let mut sas_port = String::new();
        if let Ok(link) = fs::read_link(phydevicepath.join("port")) {
            if SAS_PORT_DEVICE_REGEX.is_match(link.file_name().unwrap().to_str().unwrap()) {
                sas_port = link.file_name().unwrap().to_str().unwrap().to_string();
            }
        }

        let files = fs::read_dir(&phypath)?;
        let mut phy = SASPhy {
            name: name.to_string(),
            sas_address: String::new(),
            sas_port,
            device_type: String::new(),
            initiator_port_protocols: Vec::new(),
            invalid_dword_count: 0,
            loss_of_dword_sync_count: 0,
            maximum_linkrate: 0.0,
            maximum_linkrate_hw: 0.0,
            minimum_linkrate: 0.0,
            minimum_linkrate_hw: 0.0,
            negotiated_linkrate: 0.0,
            phy_identifier: String::new(),
            phy_reset_problem_count: 0,
            running_disparity_error_count: 0,
            target_port_protocols: Vec::new(),
        };

        for file in files {
            let file = file?;
            if !file.file_type()?.is_file() {
                continue;
            }

            let name = file.file_name().to_string_lossy().to_string();
            let value = fs::read_to_string(file.path())?.trim().to_string();

            match name.as_str() {
                "sas_address" => phy.sas_address = value,
                "device_type" => phy.device_type = value,
                "initiator_port_protocols" => phy.initiator_port_protocols = value.split(", ").map(String::from).collect(),
                "invalid_dword_count" => phy.invalid_dword_count = value.parse().unwrap_or(0),
                "loss_of_dword_sync_count" => phy.loss_of_dword_sync_count = value.parse().unwrap_or(0),
                "maximum_linkrate" => phy.maximum_linkrate = parse_linkrate(&value),
                "maximum_linkrate_hw" => phy.maximum_linkrate_hw = parse_linkrate(&value),
                "minimum_linkrate" => phy.minimum_linkrate = parse_linkrate(&value),
                "minimum_linkrate_hw" => phy.minimum_linkrate_hw = parse_linkrate(&value),
                "negotiated_linkrate" => phy.negotiated_linkrate = parse_linkrate(&value),
                "phy_identifier" => phy.phy_identifier = value,
                "phy_reset_problem_count" => phy.phy_reset_problem_count = value.parse().unwrap_or(0),
                "running_disparity_error_count" => phy.running_disparity_error_count = value.parse().unwrap_or(0),
                "target_port_protocols" => phy.target_port_protocols = value.split(", ").map(String::from).collect(),
                _ => {}
            }
        }

        Ok(phy)
    }
}

fn parse_linkrate(value: &str) -> f64 {
    let parts: Vec<&str> = value.split_whitespace().collect();
    if parts.len() < 1 {
        return 0.0;
    }
    parts[0].parse::<f64>().unwrap_or(0.0)
}

impl SASPhyClass {
    pub fn get_by_name(&self, name: &str) -> Option<&SASPhy> {
        self.get(name)
    }
}