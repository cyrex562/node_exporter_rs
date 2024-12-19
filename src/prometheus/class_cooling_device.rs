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
use std::io::{self, Read};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
pub struct ClassCoolingDeviceStats {
    name: String,   // The name of the cooling device.
    device_type: String, // Type of the cooling device (processor/fan/...)
    max_state: i64, // Maximum cooling state of the cooling device
    cur_state: i64, // Current cooling state of the cooling device
}

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn class_cooling_device_stats(&self) -> Result<Vec<ClassCoolingDeviceStats>, io::Error> {
        let pattern = format!("{}/class/thermal/cooling_device[0-9]*", self.sys_path);
        let paths = glob::glob(&pattern)?;

        let mut stats = Vec::new();
        for entry in paths {
            let path = entry?;
            let cd_name = path.file_name().unwrap().to_str().unwrap().trim_start_matches("cooling_device").to_string();
            let cooling_device_stats = parse_cooling_device_stats(&path)?;
            stats.push(ClassCoolingDeviceStats {
                name: cd_name,
                ..cooling_device_stats
            });
        }
        Ok(stats)
    }
}

fn parse_cooling_device_stats(cd: &Path) -> Result<ClassCoolingDeviceStats, io::Error> {
    let cd_type = fs::read_to_string(cd.join("type"))?.trim().to_string();

    let cd_max_state_string = fs::read_to_string(cd.join("max_state"))?.trim().to_string();
    let cd_max_state_int = i64::from_str(&cd_max_state_string)?;

    let cd_cur_state_string = fs::read_to_string(cd.join("cur_state"))?.trim().to_string();
    let cd_cur_state_int = i64::from_str(&cd_cur_state_string)?;

    Ok(ClassCoolingDeviceStats {
        name: String::new(),
        device_type: cd_type,
        max_state: cd_max_state_int,
        cur_state: cd_cur_state_int,
    })
}