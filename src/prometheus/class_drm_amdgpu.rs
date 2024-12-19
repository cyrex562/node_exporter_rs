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
use regex::Regex;
use std::collections::HashMap;

const DEVICE_DRIVER_AMDGPU: &str = "amdgpu";

#[derive(Debug, Default)]
pub struct ClassDRMCardAMDGPUStats {
    name: String,
    gpu_busy_percent: Option<u64>,
    memory_gtt_size: Option<u64>,
    memory_gtt_used: Option<u64>,
    memory_visible_vram_size: Option<u64>,
    memory_visible_vram_used: Option<u64>,
    memory_vram_size: Option<u64>,
    memory_vram_used: Option<u64>,
    memory_vram_vendor: Option<String>,
    power_dpm_force_performance_level: Option<String>,
    unique_id: Option<String>,
}

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn class_drm_card_amdgpu_stats(&self) -> Result<Vec<ClassDRMCardAMDGPUStats>, io::Error> {
        let pattern = format!("{}/class/drm/card[0-9]", self.sys_path);
        let paths = glob::glob(&pattern)?;

        let mut stats = Vec::new();
        for entry in paths {
            let path = entry?;
            let card_stats = parse_class_drm_amdgpu_card(&path)?;
            if let Some(mut card_stats) = card_stats {
                card_stats.name = path.file_name().unwrap().to_str().unwrap().to_string();
                stats.push(card_stats);
            }
        }
        Ok(stats)
    }
}

fn parse_class_drm_amdgpu_card(card: &Path) -> Result<Option<ClassDRMCardAMDGPUStats>, io::Error> {
    let uevent = fs::read_to_string(card.join("device/uevent"))?;
    let re = Regex::new(&format!("DRIVER={}", DEVICE_DRIVER_AMDGPU)).unwrap();
    if !re.is_match(&uevent) {
        return Ok(None);
    }

    let mut stats = ClassDRMCardAMDGPUStats::default();
    stats.gpu_busy_percent = read_drm_card_field(card, "gpu_busy_percent")?.and_then(|v| v.parse().ok());
    stats.memory_gtt_size = read_drm_card_field(card, "mem_info_gtt_total")?.and_then(|v| v.parse().ok());
    stats.memory_gtt_used = read_drm_card_field(card, "mem_info_gtt_used")?.and_then(|v| v.parse().ok());
    stats.memory_visible_vram_size = read_drm_card_field(card, "mem_info_vis_vram_total")?.and_then(|v| v.parse().ok());
    stats.memory_visible_vram_used = read_drm_card_field(card, "mem_info_vis_vram_used")?.and_then(|v| v.parse().ok());
    stats.memory_vram_size = read_drm_card_field(card, "mem_info_vram_total")?.and_then(|v| v.parse().ok());
    stats.memory_vram_used = read_drm_card_field(card, "mem_info_vram_used")?.and_then(|v| v.parse().ok());
    stats.memory_vram_vendor = read_drm_card_field(card, "mem_info_vram_vendor")?;
    stats.power_dpm_force_performance_level = read_drm_card_field(card, "power_dpm_force_performance_level")?;
    stats.unique_id = read_drm_card_field(card, "unique_id")?;

    Ok(Some(stats))
}

fn read_drm_card_field(card: &Path, field: &str) -> Result<Option<String>, io::Error> {
    let path = card.join(field);
    match fs::read_to_string(&path) {
        Ok(value) => Ok(Some(value.trim().to_string())),
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(e)
            }
        }
    }
}