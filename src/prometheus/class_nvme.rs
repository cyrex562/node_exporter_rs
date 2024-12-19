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

use std::fs;
use std::io;
use std::path::Path;
use std::collections::HashMap;

const NVME_CLASS_PATH: &str = "class/nvme";

#[derive(Debug, Default)]
pub struct NVMeDevice {
    name: String,
    serial: String,
    model: String,
    state: String,
    firmware_revision: String,
}

pub type NVMeClass = HashMap<String, NVMeDevice>;

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn nvme_class(&self) -> Result<NVMeClass, io::Error> {
        let path = Path::new(&self.sys_path).join(NVME_CLASS_PATH);
        let dirs = fs::read_dir(path)?;

        let mut nc = NVMeClass::new();
        for dir in dirs {
            let dir = dir?;
            let device = self.parse_nvme_device(&dir.file_name().to_string_lossy())?;
            nc.insert(device.name.clone(), device);
        }

        Ok(nc)
    }

    fn parse_nvme_device(&self, name: &str) -> Result<NVMeDevice, io::Error> {
        let path = Path::new(&self.sys_path).join(NVME_CLASS_PATH).join(name);
        let mut device = NVMeDevice {
            name: name.to_string(),
            ..Default::default()
        };

        for field in &["firmware_rev", "model", "serial", "state"] {
            let value = read_file_to_string(&path.join(field))?;
            match *field {
                "firmware_rev" => device.firmware_revision = value,
                "model" => device.model = value,
                "serial" => device.serial = value,
                "state" => device.state = value,
                _ => {}
            }
        }

        Ok(device)
    }
}

fn read_file_to_string(path: &Path) -> Result<String, io::Error> {
    fs::read_to_string(path).map(|s| s.trim().to_string())
}