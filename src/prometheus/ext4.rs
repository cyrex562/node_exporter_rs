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

// Package btrfs provides access to statistics exposed by ext4 filesystems.
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SYS_FS_PATH: &str = "fs";
const SYS_FS_EXT4_PATH: &str = "ext4";

#[derive(Default)]
pub struct Stats {
    name: String,
    errors: u64,
    warnings: u64,
    messages: u64,
}

pub struct FS {
    proc: PathBuf,
    sys: PathBuf,
}

impl FS {
    pub fn new_default_fs() -> io::Result<Self> {
        Self::new_fs("/proc", "/sys")
    }

    pub fn new_fs(proc_mount_point: &str, sys_mount_point: &str) -> io::Result<Self> {
        let proc = if proc_mount_point.trim().is_empty() {
            PathBuf::from("/proc")
        } else {
            PathBuf::from(proc_mount_point)
        };

        let sys = if sys_mount_point.trim().is_empty() {
            PathBuf::from("/sys")
        } else {
            PathBuf::from(sys_mount_point)
        };

        Ok(FS { proc, sys })
    }

    pub fn proc_stat(&self) -> io::Result<Vec<Stats>> {
        let pattern = self.sys.join(SYS_FS_PATH).join(SYS_FS_EXT4_PATH).join("*");
        let matches = glob::glob(pattern.to_str().unwrap())?;

        let mut stats = Vec::new();
        for entry in matches {
            let path = entry?;
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            let mut s = Stats {
                name,
                ..Default::default()
            };

            for (file, field) in [
                ("errors_count", &mut s.errors),
                ("warning_count", &mut s.warnings),
                ("msg_count", &mut s.messages),
            ] {
                let file_path = self.sys.join(SYS_FS_PATH).join(SYS_FS_EXT4_PATH).join(&s.name).join(file);
                if let Ok(val) = read_uint_from_file(&file_path) {
                    *field = val;
                }
            }

            stats.push(s);
        }

        Ok(stats)
    }
}

fn read_uint_from_file(path: &Path) -> io::Result<u64> {
    let content = fs::read_to_string(path)?;
    content.trim().parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}