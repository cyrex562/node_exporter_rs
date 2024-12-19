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

use std::fs;
use std::io;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
pub struct ClassThermalZoneStats {
    name: String,
    type_: String,
    temp: i64,
    policy: String,
    mode: Option<bool>,
    passive: Option<u64>,
}

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn class_thermal_zone_stats(&self) -> Result<Vec<ClassThermalZoneStats>, io::Error> {
        let pattern = format!("{}/class/thermal/thermal_zone[0-9]*", self.sys_path);
        let zones = glob::glob(&pattern)?;

        let mut stats = Vec::new();
        for zone in zones {
            let zone = zone?;
            match parse_class_thermal_zone(&zone) {
                Ok(mut zone_stats) => {
                    zone_stats.name = zone.file_name().unwrap().to_str().unwrap().trim_start_matches("thermal_zone").to_string();
                    stats.push(zone_stats);
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound || e.kind() == io::ErrorKind::PermissionDenied => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(stats)
    }
}

fn parse_class_thermal_zone(zone: &Path) -> Result<ClassThermalZoneStats, io::Error> {
    let zone_type = fs::read_to_string(zone.join("type"))?.trim().to_string();
    let zone_policy = fs::read_to_string(zone.join("policy"))?.trim().to_string();
    let zone_temp = fs::read_to_string(zone.join("temp"))?.trim().parse::<i64>()?;

    let mode = match fs::read_to_string(zone.join("mode")) {
        Ok(value) => Some(value.trim().eq_ignore_ascii_case("enabled")),
        Err(e) if e.kind() == io::ErrorKind::NotFound || e.kind() == io::ErrorKind::PermissionDenied => None,
        Err(e) => return Err(e),
    };

    let passive = match fs::read_to_string(zone.join("passive")) {
        Ok(value) => Some(value.trim().parse::<u64>()?),
        Err(e) if e.kind() == io::ErrorKind::NotFound || e.kind() == io::ErrorKind::PermissionDenied => None,
        Err(e) => return Err(e),
    };

    Ok(ClassThermalZoneStats {
        name: String::new(),
        type_: zone_type,
        temp: zone_temp,
        policy: zone_policy,
        mode,
        passive,
    })
}