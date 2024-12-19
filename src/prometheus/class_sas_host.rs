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

const SAS_HOST_CLASS_PATH: &str = "class/sas_host";

#[derive(Debug)]
pub struct SASHost {
    name: String,
    sas_phys: Vec<String>,
    sas_ports: Vec<String>,
}

pub type SASHostClass = HashMap<String, SASHost>;

lazy_static::lazy_static! {
    static ref SAS_PHY_DEVICE_REGEX: Regex = Regex::new(r"^phy-[0-9:]+$").unwrap();
    static ref SAS_PORT_DEVICE_REGEX: Regex = Regex::new(r"^port-[0-9:]+$").unwrap();
}

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn sas_host_class(&self) -> Result<SASHostClass, io::Error> {
        let path = Path::new(&self.sys_path).join(SAS_HOST_CLASS_PATH);
        let dirs = fs::read_dir(path)?;

        let mut shc = SASHostClass::new();
        for dir in dirs {
            let dir = dir?;
            let host = self.parse_sas_host(&dir.file_name().to_string_lossy())?;
            shc.insert(host.name.clone(), host);
        }

        Ok(shc)
    }

    fn parse_sas_host(&self, name: &str) -> Result<SASHost, io::Error> {
        let device_path = Path::new(&self.sys_path).join(SAS_HOST_CLASS_PATH).join(name).join("device");
        let dirs = fs::read_dir(&device_path)?;

        let mut sas_phys = Vec::new();
        let mut sas_ports = Vec::new();
        for dir in dirs {
            let dir = dir?;
            let dir_name = dir.file_name().to_string_lossy().to_string();
            if SAS_PHY_DEVICE_REGEX.is_match(&dir_name) {
                sas_phys.push(dir_name.clone());
            }
            if SAS_PORT_DEVICE_REGEX.is_match(&dir_name) {
                sas_ports.push(dir_name.clone());
            }
        }

        Ok(SASHost {
            name: name.to_string(),
            sas_phys,
            sas_ports,
        })
    }
}

impl SASHostClass {
    pub fn get_by_name(&self, name: &str) -> Option<&SASHost> {
        self.get(name)
    }

    pub fn get_by_phy(&self, phy_name: &str) -> Option<&SASHost> {
        self.values().find(|h| h.sas_phys.contains(&phy_name.to_string()))
    }

    pub fn get_by_port(&self, port_name: &str) -> Option<&SASHost> {
        self.values().find(|h| h.sas_ports.contains(&port_name.to_string()))
    }
}