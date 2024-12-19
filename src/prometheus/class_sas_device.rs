// Copyright 2022 The Prometheus Authors
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

const SAS_DEVICE_CLASS_PATH: &str = "class/sas_device";
const SAS_END_DEVICE_CLASS_PATH: &str = "class/sas_end_device";
const SAS_EXPANDER_CLASS_PATH: &str = "class/sas_expander";

#[derive(Debug)]
pub struct SASDevice {
    name: String,
    sas_address: String,
    sas_phys: Vec<String>,
    sas_ports: Vec<String>,
    block_devices: Vec<String>,
}

pub type SASDeviceClass = HashMap<String, SASDevice>;

lazy_static::lazy_static! {
    static ref SAS_TARGET_DEVICE_REGEX: Regex = Regex::new(r"^target[0-9:]+$").unwrap();
    static ref SAS_TARGET_SUB_DEVICE_REGEX: Regex = Regex::new(r"[0-9]+:.*").unwrap();
}

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn sas_device_class(&self) -> Result<SASDeviceClass, io::Error> {
        self.parse_sas_device_class(SAS_DEVICE_CLASS_PATH)
    }

    pub fn sas_end_device_class(&self) -> Result<SASDeviceClass, io::Error> {
        self.parse_sas_device_class(SAS_END_DEVICE_CLASS_PATH)
    }

    pub fn sas_expander_class(&self) -> Result<SASDeviceClass, io::Error> {
        self.parse_sas_device_class(SAS_EXPANDER_CLASS_PATH)
    }

    fn parse_sas_device_class(&self, dir: &str) -> Result<SASDeviceClass, io::Error> {
        let path = Path::new(&self.sys_path).join(dir);
        let dirs = fs::read_dir(path)?;

        let mut sdc = SASDeviceClass::new();
        for dir in dirs {
            let dir = dir?;
            let device = self.parse_sas_device(&dir.file_name().to_string_lossy())?;
            sdc.insert(device.name.clone(), device);
        }

        Ok(sdc)
    }

    fn parse_sas_device(&self, name: &str) -> Result<SASDevice, io::Error> {
        let device_path = Path::new(&self.sys_path).join(SAS_DEVICE_CLASS_PATH).join(name).join("device");
        let dirs = fs::read_dir(&device_path)?;

        let mut sas_phys = Vec::new();
        let mut sas_ports = Vec::new();
        for dir in dirs {
            let dir = dir?;
            let dir_name = dir.file_name().to_string_lossy().to_string();
            if SAS_TARGET_DEVICE_REGEX.is_match(&dir_name) {
                sas_phys.push(dir_name.clone());
            }
            if SAS_TARGET_SUB_DEVICE_REGEX.is_match(&dir_name) {
                sas_ports.push(dir_name.clone());
            }
        }

        let sas_address = fs::read_to_string(Path::new(&self.sys_path).join(SAS_DEVICE_CLASS_PATH).join(name).join("sas_address"))?.trim().to_string();
        let block_devices = self.block_sas_device_block_devices(name)?;

        Ok(SASDevice {
            name: name.to_string(),
            sas_address,
            sas_phys,
            sas_ports,
            block_devices,
        })
    }

    fn block_sas_device_block_devices(&self, name: &str) -> Result<Vec<String>, io::Error> {
        let device_path = Path::new(&self.sys_path).join(SAS_DEVICE_CLASS_PATH).join(name).join("device");
        let dirs = fs::read_dir(&device_path)?;

        let mut devices = Vec::new();
        for dir in dirs {
            let dir = dir?;
            if SAS_TARGET_DEVICE_REGEX.is_match(&dir.file_name().to_string_lossy()) {
                let target_dir = dir.file_name().to_string_lossy().to_string();
                let subtargets = fs::read_dir(device_path.join(&target_dir))?;
                for subtarget in subtargets {
                    let subtarget = subtarget?;
                    if SAS_TARGET_SUB_DEVICE_REGEX.is_match(&subtarget.file_name().to_string_lossy()) {
                        let blocks = fs::read_dir(device_path.join(&target_dir).join(subtarget.file_name()))?;
                        for block in blocks {
                            let block = block?;
                            devices.push(block.file_name().to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        Ok(devices)
    }
}

impl SASDeviceClass {
    pub fn get_by_name(&self, name: &str) -> Option<&SASDevice> {
        self.get(name)
    }

    pub fn get_by_phy(&self, name: &str) -> Option<&SASDevice> {
        self.values().find(|d| d.sas_phys.contains(&name.to_string()))
    }

    pub fn get_by_port(&self, name: &str) -> Option<&SASDevice> {
        self.values().find(|d| d.sas_ports.contains(&name.to_string()))
    }
}