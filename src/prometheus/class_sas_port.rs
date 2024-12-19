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

const SAS_PORT_CLASS_PATH: &str = "class/sas_port";

#[derive(Debug)]
pub struct SASPort {
    name: String,
    sas_phys: Vec<String>,
    expanders: Vec<String>,
    end_devices: Vec<String>,
}

pub type SASPortClass = HashMap<String, SASPort>;

lazy_static::lazy_static! {
    static ref SAS_PHY_DEVICE_REGEX: Regex = Regex::new(r"^phy-[0-9:]+$").unwrap();
    static ref SAS_EXPANDER_DEVICE_REGEX: Regex = Regex::new(r"^expander-[0-9:]+$").unwrap();
    static ref SAS_END_DEVICE_REGEX: Regex = Regex::new(r"^end_device-[0-9:]+$").unwrap();
}

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn sas_port_class(&self) -> Result<SASPortClass, io::Error> {
        let path = Path::new(&self.sys_path).join(SAS_PORT_CLASS_PATH);
        let dirs = fs::read_dir(path)?;

        let mut spc = SASPortClass::new();
        for dir in dirs {
            let dir = dir?;
            let port = self.parse_sas_port(&dir.file_name().to_string_lossy())?;
            spc.insert(port.name.clone(), port);
        }

        Ok(spc)
    }

    fn parse_sas_port(&self, name: &str) -> Result<SASPort, io::Error> {
        let port_path = Path::new(&self.sys_path).join(SAS_PORT_CLASS_PATH).join(name).join("device");
        let dirs = fs::read_dir(&port_path)?;

        let mut sas_phys = Vec::new();
        let mut expanders = Vec::new();
        let mut end_devices = Vec::new();
        for dir in dirs {
            let dir = dir?;
            let dir_name = dir.file_name().to_string_lossy().to_string();
            if SAS_PHY_DEVICE_REGEX.is_match(&dir_name) {
                sas_phys.push(dir_name.clone());
            }
            if SAS_EXPANDER_DEVICE_REGEX.is_match(&dir_name) {
                expanders.push(dir_name.clone());
            }
            if SAS_END_DEVICE_REGEX.is_match(&dir_name) {
                end_devices.push(dir_name.clone());
            }
        }

        Ok(SASPort {
            name: name.to_string(),
            sas_phys,
            expanders,
            end_devices,
        })
    }
}

impl SASPortClass {
    pub fn get_by_name(&self, name: &str) -> Option<&SASPort> {
        self.get(name)
    }

    pub fn get_by_phy(&self, name: &str) -> Option<&SASPort> {
        self.values().find(|d| d.sas_phys.contains(&name.to_string()))
    }

    pub fn get_by_expander(&self, name: &str) -> Option<&SASPort> {
        self.values().find(|d| d.expanders.contains(&name.to_string()))
    }

    pub fn get_by_end_device(&self, name: &str) -> Option<&SASPort> {
        self.values().find(|d| d.end_devices.contains(&name.to_string()))
    }
}