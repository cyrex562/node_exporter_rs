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

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
pub struct RaplZone {
    name: String,
    index: usize,
    path: String,
    max_microjoules: u64,
}

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn get_rapl_zones(&self) -> Result<Vec<RaplZone>, io::Error> {
        let rapl_dir = Path::new(&self.sys_path).join("class/powercap");
        let entries = fs::read_dir(rapl_dir)?;

        let mut zones = Vec::new();
        let mut count_name_usages = HashMap::new();

        for entry in entries {
            let entry = entry?;
            let name_file = entry.path().join("name");
            if let Ok(name_bytes) = fs::read_to_string(&name_file) {
                let name = name_bytes.trim().to_string();
                let (index, name) = get_index_and_name(&mut count_name_usages, &name);

                let max_microjoule_file = entry.path().join("max_energy_range_uj");
                let max_microjoules = read_u64_from_file(&max_microjoule_file)?;

                let zone = RaplZone {
                    name: name.clone(),
                    index,
                    path: entry.path().to_string_lossy().to_string(),
                    max_microjoules,
                };

                zones.push(zone);
                count_name_usages.insert(name, index + 1);
            }
        }

        Ok(zones)
    }
}

impl RaplZone {
    pub fn get_energy_microjoules(&self) -> Result<u64, io::Error> {
        let energy_file = Path::new(&self.path).join("energy_uj");
        read_u64_from_file(&energy_file)
    }
}

fn get_index_and_name(count_name_usages: &mut HashMap<String, usize>, name: &str) -> (usize, String) {
    let parts: Vec<&str> = name.split('-').collect();
    if parts.len() == 2 {
        if let Ok(index) = usize::from_str(parts[1]) {
            return (index, parts[0].to_string());
        }
    }
    let index = *count_name_usages.get(name).unwrap_or(&0);
    (index, name.to_string())
}

fn read_u64_from_file(path: &Path) -> Result<u64, io::Error> {
    let content = fs::read_to_string(path)?;
    content.trim().parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}