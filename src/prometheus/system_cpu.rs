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
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct CPU(String);

impl CPU {
    pub fn number(&self) -> String {
        self.0.trim_start_matches("cpu").to_string()
    }

    pub async fn topology(&self) -> Result<CPUTopology, io::Error> {
        let cpu_topology_path = Path::new(&self.0).join("topology");
        if !cpu_topology_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Topology path not found"));
        }
        parse_cpu_topology(&cpu_topology_path).await
    }

    pub async fn thermal_throttle(&self) -> Result<CPUThermalThrottle, io::Error> {
        let cpu_path = Path::new(&self.0).join("thermal_throttle");
        if !cpu_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Thermal throttle path not found"));
        }
        parse_cpu_thermal_throttle(&cpu_path).await
    }

    pub async fn online(&self) -> Result<bool, io::Error> {
        let cpu_path = Path::new(&self.0).join("online");
        let content = fs::read_to_string(cpu_path).await?;
        Ok(content.trim() == "1")
    }
}

#[derive(Debug)]
pub struct CPUTopology {
    core_id: String,
    core_siblings_list: String,
    physical_package_id: String,
    thread_siblings_list: String,
}

#[derive(Debug)]
pub struct CPUThermalThrottle {
    core_throttle_count: u64,
    package_throttle_count: u64,
}

#[derive(Debug)]
pub struct SystemCPUCpufreqStats {
    name: String,
    cpuinfo_current_frequency: Option<u64>,
    cpuinfo_minimum_frequency: Option<u64>,
    cpuinfo_maximum_frequency: Option<u64>,
    cpuinfo_transition_latency: Option<u64>,
    scaling_current_frequency: Option<u64>,
    scaling_minimum_frequency: Option<u64>,
    scaling_maximum_frequency: Option<u64>,
    available_governors: String,
    driver: String,
    governor: String,
    related_cpus: String,
    set_speed: String,
    cpuinfo_frequency_duration: Option<HashMap<u64, u64>>,
    cpuinfo_frequency_transitions_total: Option<u64>,
    cpuinfo_transition_table: Option<Vec<Vec<u64>>>,
}

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub async fn cpus(&self) -> Result<Vec<CPU>, io::Error> {
        let pattern = format!("{}/devices/system/cpu/cpu[0-9]*", self.sys_path);
        let cpu_paths = glob::glob(&pattern)?;

        let mut cpus = Vec::new();
        for cpu_path in cpu_paths {
            let cpu_path = cpu_path?;
            cpus.push(CPU(cpu_path.to_string_lossy().to_string()));
        }

        Ok(cpus)
    }

    pub async fn system_cpufreq(&self) -> Result<Vec<SystemCPUCpufreqStats>, io::Error> {
        let cpus = self.cpus().await?;
        let mut system_cpufreq = Vec::new();
        let mut handles = vec![];

        for cpu in cpus {
            let cpu_cpufreq_path = Path::new(&cpu.0).join("cpufreq");
            if !cpu_cpufreq_path.exists() {
                continue;
            }

            let cpu_name = cpu.number();
            let handle = tokio::spawn(async move {
                let cpufreq = parse_cpufreq_cpuinfo(&cpu_cpufreq_path).await?;
                Ok(SystemCPUCpufreqStats {
                    name: cpu_name,
                    ..cpufreq
                })
            });
            handles.push(handle);
        }

        for handle in handles {
            match handle.await {
                Ok(Ok(cpufreq)) => system_cpufreq.push(cpufreq),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
            }
        }

        if system_cpufreq.is_empty() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "could not find any cpufreq files"));
        }

        Ok(system_cpufreq)
    }

    pub async fn isolated_cpus(&self) -> Result<Vec<u16>, io::Error> {
        let isolcpus = fs::read_to_string(Path::new(&self.sys_path).join("devices/system/cpu/isolated")).await?;
        parse_cpu_range(&isolcpus)
    }
}

async fn parse_cpu_topology(cpu_path: &Path) -> Result<CPUTopology, io::Error> {
    let core_id = fs::read_to_string(cpu_path.join("core_id")).await?.trim().to_string();
    let physical_package_id = fs::read_to_string(cpu_path.join("physical_package_id")).await?.trim().to_string();
    let core_siblings_list = fs::read_to_string(cpu_path.join("core_siblings_list")).await?.trim().to_string();
    let thread_siblings_list = fs::read_to_string(cpu_path.join("thread_siblings_list")).await?.trim().to_string();

    Ok(CPUTopology {
        core_id,
        physical_package_id,
        core_siblings_list,
        thread_siblings_list,
    })
}

async fn parse_cpu_thermal_throttle(cpu_path: &Path) -> Result<CPUThermalThrottle, io::Error> {
    let package_throttle_count = read_u64_from_file(&cpu_path.join("package_throttle_count")).await?;
    let core_throttle_count = read_u64_from_file(&cpu_path.join("core_throttle_count")).await?;

    Ok(CPUThermalThrottle {
        package_throttle_count,
        core_throttle_count,
    })
}

async fn parse_cpufreq_cpuinfo(cpu_path: &Path) -> Result<SystemCPUCpufreqStats, io::Error> {
    let uint_files = [
        "cpuinfo_cur_freq",
        "cpuinfo_max_freq",
        "cpuinfo_min_freq",
        "cpuinfo_transition_latency",
        "scaling_cur_freq",
        "scaling_max_freq",
        "scaling_min_freq",
    ];
    let mut uint_out = vec![None; uint_files.len()];

    for (i, file) in uint_files.iter().enumerate() {
        if let Ok(value) = read_u64_from_file(&cpu_path.join(file)).await {
            uint_out[i] = Some(value);
        }
    }

    let string_files = [
        "scaling_available_governors",
        "scaling_driver",
        "scaling_governor",
        "related_cpus",
        "scaling_setspeed",
    ];
    let mut string_out = vec![String::new(); string_files.len()];

    for (i, file) in string_files.iter().enumerate() {
        string_out[i] = fs::read_to_string(cpu_path.join(file)).await?.trim().to_string();
    }

    let cpuinfo_frequency_transitions_total = read_u64_from_file(&cpu_path.join("stats/total_trans")).await.ok();
    let cpuinfo_frequency_duration = parse_cpuinfo_frequency_duration(&cpu_path.join("stats/time_in_state")).await?;
    let cpuinfo_transition_table = parse_cpuinfo_transition_table(&cpu_path.join("stats/trans_table")).await?;

    Ok(SystemCPUCpufreqStats {
        cpuinfo_current_frequency: uint_out[0],
        cpuinfo_maximum_frequency: uint_out[1],
        cpuinfo_minimum_frequency: uint_out[2],
        cpuinfo_transition_latency: uint_out[3],
        scaling_current_frequency: uint_out[4],
        scaling_maximum_frequency: uint_out[5],
        scaling_minimum_frequency: uint_out[6],
        available_governors: string_out[0].clone(),
        driver: string_out[1].clone(),
        governor: string_out[2].clone(),
        related_cpus: string_out[3].clone(),
        set_speed: string_out[4].clone(),
        cpuinfo_frequency_duration,
        cpuinfo_frequency_transitions_total,
        cpuinfo_transition_table,
    })
}

async fn parse_cpuinfo_frequency_duration(path: &Path) -> Result<Option<HashMap<u64, u64>>, io::Error> {
    let content = match fs::read_to_string(path).await {
        Ok(content) => content,
        Err(e) if e.kind() == io::ErrorKind::NotFound || e.kind() == io::ErrorKind::PermissionDenied => return Ok(None),
        Err(e) => return Err(e),
    };

    let mut map = HashMap::new();
    for line in content.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() != 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected number of fields in time_in_state"));
        }
        let freq = u64::from_str(fields[0])?;
        let duration = u64::from_str(fields[1])?;
        map.insert(freq, duration);
    }

    Ok(Some(map))
}

async fn parse_cpuinfo_transition_table(path: &Path) -> Result<Option<Vec<Vec<u64>>>, io::Error> {
    let content = match fs::read_to_string(path).await {
        Ok(content) => content,
        Err(e) if e.kind() == io::ErrorKind::NotFound || e.kind() == io::ErrorKind::PermissionDenied => return Ok(None),
        Err(e) => return Err(e),
    };

    let mut table = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if i == 0 || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        let row: Result<Vec<u64>, _> = fields.iter().map(|&f| u64::from_str(f)).collect();
        table.push(row?);
    }

    Ok(Some(table))
}

async fn read_u64_from_file(path: &Path) -> Result<u64, io::Error> {
    let content = fs::read_to_string(path).await?;
    content.trim().parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn parse_cpu_range(data: &str) -> Result<Vec<u16>, io::Error> {
    let mut cpus = Vec::new();

    for cpu in data.trim().split(',') {
        if cpu.contains('-') {
            let ranges: Vec<&str> = cpu.split('-').collect();
            if ranges.len() != 2 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("invalid cpu range: {}", cpu)));
            }
            let start_range = u16::from_str(ranges[0])?;
            let end_range = u16::from_str(ranges[1])?;
            cpus.extend(start_range..=end_range);
        } else {
            cpus.push(u16::from_str(cpu)?);
        }
    }

    Ok(cpus)
}