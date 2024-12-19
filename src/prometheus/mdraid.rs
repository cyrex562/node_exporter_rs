// Copyright 2023 The Prometheus Authors
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
pub struct Mdraid {
    device: String,
    level: String,
    array_state: String,
    metadata_version: String,
    disks: u64,
    components: Vec<MdraidComponent>,
    uuid: String,
    chunk_size: Option<u64>,
    degraded_disks: Option<u64>,
    sync_action: Option<String>,
    sync_completed: Option<f64>,
}

#[derive(Debug)]
pub struct MdraidComponent {
    device: String,
    state: String,
}

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn mdraids(&self) -> Result<Vec<Mdraid>, io::Error> {
        let pattern = format!("{}/block/md*/md", self.sys_path);
        let matches = glob::glob(&pattern)?;

        let mut mdraids = Vec::new();
        for m in matches {
            let m = m?;
            let device = m.parent().unwrap().file_name().unwrap().to_str().unwrap().to_string();
            let path = Path::new(&self.sys_path).join("block").join(&device).join("md");

            let level = read_file_to_string(&path.join("level"))?;
            let array_state = read_file_to_string(&path.join("array_state"))?;
            let metadata_version = read_file_to_string(&path.join("metadata_version"))?;
            let disks = read_file_to_u64(&path.join("raid_disks"))?;
            let uuid = read_file_to_string(&path.join("uuid"))?;

            let mut components = Vec::new();
            let devs = glob::glob(&format!("{}/dev-*", path.display()))?;
            for dev in devs {
                let dev = dev?;
                let device = dev.file_name().unwrap().to_str().unwrap().trim_start_matches("dev-").to_string();
                let state = read_file_to_string(&dev.join("state"))?;
                components.push(MdraidComponent { device, state });
            }

            let chunk_size = if ["raid0", "raid4", "raid5", "raid6", "raid10"].contains(&level.as_str()) {
                Some(read_file_to_u64(&path.join("chunk_size"))?)
            } else {
                None
            };

            let degraded_disks = if ["raid1", "raid4", "raid5", "raid6", "raid10"].contains(&level.as_str()) {
                Some(read_file_to_u64(&path.join("degraded"))?)
            } else {
                None
            };

            let sync_action = if ["raid1", "raid4", "raid5", "raid6", "raid10"].contains(&level.as_str()) {
                Some(read_file_to_string(&path.join("sync_action"))?)
            } else {
                None
            };

            let sync_completed = if ["raid1", "raid4", "raid5", "raid6", "raid10"].contains(&level.as_str()) {
                let val = read_file_to_string(&path.join("sync_completed"))?;
                if val != "none" {
                    let parts: Vec<&str> = val.split(" / ").collect();
                    if parts.len() == 2 {
                        let a = u64::from_str(parts[0].trim())?;
                        let b = u64::from_str(parts[1].trim())?;
                        Some(a as f64 / b as f64)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            mdraids.push(Mdraid {
                device,
                level,
                array_state,
                metadata_version,
                disks,
                components,
                uuid,
                chunk_size,
                degraded_disks,
                sync_action,
                sync_completed,
            });
        }

        Ok(mdraids)
    }
}

fn read_file_to_string(path: &Path) -> Result<String, io::Error> {
    fs::read_to_string(path).map(|s| s.trim().to_string())
}

fn read_file_to_u64(path: &Path) -> Result<u64, io::Error> {
    fs::read_to_string(path)?.trim().parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}