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

//! Provides access to statistics exposed by Btrfs filesystems.

use std::collections::HashMap;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Contains statistics for a single Btrfs filesystem.
/// See Linux fs/btrfs/sysfs.c for more information.
pub struct Stats {
    pub uuid: String,
    pub label: String,
    pub allocation: Allocation,
    pub devices: HashMap<String, Device>,
    pub features: Vec<String>,
    pub clone_alignment: u64,
    pub node_size: u64,
    pub quota_override: u64,
    pub sector_size: u64,
    pub commit_stats: CommitStats,
}

/// Contains allocation statistics for data, metadata and system data.
pub struct Allocation {
    pub global_rsv_reserved: u64,
    pub global_rsv_size: u64,
    pub data: AllocationStats,
    pub metadata: AllocationStats,
    pub system: AllocationStats,
}

/// Contains allocation statistics for a data type.
pub struct AllocationStats {
    // Usage statistics
    pub disk_used_bytes: u64,
    pub disk_total_bytes: u64,
    pub may_use_bytes: u64,
    pub pinned_bytes: u64,
    pub total_pinned_bytes: u64,
    pub read_only_bytes: u64,
    pub reserved_bytes: u64,
    pub used_bytes: u64,
    pub total_bytes: u64,

    // Flags marking filesystem state
    // See Linux fs/btrfs/ctree.h for more information.
    pub flags: u64,

    // Additional disk usage statistics depending on the disk layout.
    // At least one of these will exist and not be nil.
    pub layouts: HashMap<String, LayoutUsage>,
}

/// Contains additional usage statistics for a disk layout.
pub struct LayoutUsage {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub ratio: f64,
}

/// Contains information about a device that is part of a Btrfs filesystem.
pub struct Device {
    pub size: u64,
}

/// Number of commits and various time related statistics.
/// See Linux fs/btrfs/sysfs.c with 6.x version.
pub struct CommitStats {
    pub commits: u64,
    pub last_commit_ms: u64,
    pub max_commit_ms: u64,
    pub total_commit_ms: u64,
}

struct FS {
    sys: PathBuf,
}

impl FS {
    fn new_default_fs() -> io::Result<Self> {
        Self::new_fs("/sys")
    }

    fn new_fs(mount_point: &str) -> io::Result<Self> {
        let sys = PathBuf::from(mount_point);
        if !sys.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Mount point not found",
            ));
        }
        Ok(FS { sys })
    }

    fn stats(&self) -> io::Result<Vec<Stats>> {
        let mut stats = Vec::new();
        for entry in fs::read_dir(self.sys.join("fs/btrfs"))? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(stat) = get_stats(&path)? {
                    stats.push(stat);
                }
            }
        }
        Ok(stats)
    }
}

fn get_stats(uuid_path: &Path) -> io::Result<Option<Stats>> {
    let mut reader = Reader::new(uuid_path);
    let stats = reader.read_filesystem_stats();
    if reader.err.is_some() {
        return Err(reader.err.unwrap());
    }
    Ok(stats)
}

struct Reader {
    path: PathBuf,
    err: Option<io::Error>,
    dev_count: usize,
}

impl Reader {
    fn new(path: &Path) -> Self {
        Reader {
            path: path.to_path_buf(),
            err: None,
            dev_count: 0,
        }
    }

    fn read_file(&self, name: &str) -> Option<String> {
        let path = self.path.join(name);
        match fs::read_to_string(&path) {
            Ok(content) => Some(content.trim().to_string()),
            Err(e) => {
                if e.kind() != io::ErrorKind::NotFound {
                    self.err = Some(e);
                }
                None
            }
        }
    }

    fn read_value<T: FromStr>(&self, name: &str) -> Option<T> {
        self.read_file(name).and_then(|s| s.parse().ok())
    }

    fn list_files(&self, dir: &str) -> Vec<String> {
        let path = self.path.join(dir);
        match fs::read_dir(&path) {
            Ok(entries) => entries
                .filter_map(|e| e.ok().map(|e| e.file_name().into_string().unwrap()))
                .collect(),
            Err(e) => {
                self.err = Some(e);
                Vec::new()
            }
        }
    }

    fn read_allocation_stats(&self, dir: &str) -> Option<AllocationStats> {
        let sub_reader = Reader::new(&self.path.join(dir));
        Some(AllocationStats {
            may_use_bytes: sub_reader.read_value("bytes_may_use")?,
            pinned_bytes: sub_reader.read_value("bytes_pinned")?,
            read_only_bytes: sub_reader.read_value("bytes_readonly")?,
            reserved_bytes: sub_reader.read_value("bytes_reserved")?,
            used_bytes: sub_reader.read_value("bytes_used")?,
            disk_used_bytes: sub_reader.read_value("disk_used")?,
            disk_total_bytes: sub_reader.read_value("disk_total")?,
            flags: sub_reader.read_value("flags")?,
            total_bytes: sub_reader.read_value("total_bytes")?,
            total_pinned_bytes: sub_reader.read_value("total_bytes_pinned")?,
            layouts: sub_reader.read_layouts(),
        })
    }

    fn read_layouts(&self) -> HashMap<String, LayoutUsage> {
        let mut layouts = HashMap::new();
        for entry in fs::read_dir(&self.path).unwrap_or_else(|_| Vec::new()) {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    let name = entry.file_name().into_string().unwrap();
                    layouts.insert(name.clone(), self.read_layout(&name));
                }
            }
        }
        layouts
    }

    fn read_layout(&self, dir: &str) -> LayoutUsage {
        LayoutUsage {
            total_bytes: self
                .read_value(&format!("{}/total_bytes", dir))
                .unwrap_or(0),
            used_bytes: self.read_value(&format!("{}/used_bytes", dir)).unwrap_or(0),
            ratio: self.calc_ratio(dir),
        }
    }

    fn calc_ratio(&self, layout: &str) -> f64 {
        match layout {
            "single" | "raid0" => 1.0,
            "dup" | "raid1" | "raid10" => 2.0,
            "raid5" => self.dev_count as f64 / (self.dev_count as f64 - 1.0),
            "raid6" => self.dev_count as f64 / (self.dev_count as f64 - 2.0),
            _ => 0.0,
        }
    }

    fn read_device_info(&self, dir: &str) -> HashMap<String, Device> {
        let mut devices = HashMap::new();
        for name in self.list_files(dir) {
            devices.insert(
                name.clone(),
                Device {
                    size: self
                        .read_value(&format!("devices/{}/size", name))
                        .unwrap_or(0)
                        * 512,
                },
            );
        }
        devices
    }

    fn read_filesystem_stats(&mut self) -> Option<Stats> {
        let devices = self.read_device_info("devices");
        self.dev_count = devices.len();

        Some(Stats {
            label: self.read_file("label"),
            uuid: self.read_file("metadata_uuid"),
            features: self.list_files("features"),
            clone_alignment: self.read_value("clone_alignment")?,
            node_size: self.read_value("nodesize")?,
            quota_override: self.read_value("quota_override")?,
            sector_size: self.read_value("sectorsize")?,
            devices,
            allocation: Allocation {
                global_rsv_reserved: self.read_value("allocation/global_rsv_reserved")?,
                global_rsv_size: self.read_value("allocation/global_rsv_size")?,
                data: self.read_allocation_stats("allocation/data")?,
                metadata: self.read_allocation_stats("allocation/metadata")?,
                system: self.read_allocation_stats("allocation/system")?,
            },
            commit_stats: self.read_commit_stats("commit_stats"),
        })
    }

    fn read_commit_stats(&self, file: &str) -> CommitStats {
        let mut stats = CommitStats::default();
        let path = self.path.join(file);
        if let Ok(file) = fs::File::open(&path) {
            for line in io::BufReader::new(file).lines() {
                if let Ok(line) = line {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() == 2 {
                        if let Ok(value) = parts[1].parse::<u64>() {
                            match parts[0] {
                                "commits" => stats.commits = value,
                                "last_commit_ms" => stats.last_commit_ms = value,
                                "max_commit_ms" => stats.max_commit_ms = value,
                                "total_commit_ms" => stats.total_commit_ms = value,
                                _ => (),
                            }
                        }
                    }
                }
            }
        }
        stats
    }
}

#[derive(Default)]
struct Stats {
    label: Option<String>,
    uuid: Option<String>,
    features: Vec<String>,
    clone_alignment: u64,
    node_size: u64,
    quota_override: u64,
    sector_size: u64,
    devices: HashMap<String, Device>,
    allocation: Allocation,
    commit_stats: CommitStats,
}

#[derive(Default)]
struct Allocation {
    global_rsv_reserved: u64,
    global_rsv_size: u64,
    data: Option<AllocationStats>,
    metadata: Option<AllocationStats>,
    system: Option<AllocationStats>,
}

#[derive(Default)]
struct AllocationStats {
    may_use_bytes: u64,
    pinned_bytes: u64,
    read_only_bytes: u64,
    reserved_bytes: u64,
    used_bytes: u64,
    disk_used_bytes: u64,
    disk_total_bytes: u64,
    flags: u64,
    total_bytes: u64,
    total_pinned_bytes: u64,
    layouts: HashMap<String, LayoutUsage>,
}

#[derive(Default)]
struct LayoutUsage {
    total_bytes: u64,
    used_bytes: u64,
    ratio: f64,
}

#[derive(Default)]
struct Device {
    size: u64,
}

#[derive(Default)]
struct CommitStats {
    commits: u64,
    last_commit_ms: u64,
    max_commit_ms: u64,
    total_commit_ms: u64,
}
