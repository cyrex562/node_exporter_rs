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

#[cfg(target_os = "linux")]
use prometheus_procfs::fs as procfs;

pub struct FS {
    selinux: procfs::FS,
}

pub const DEFAULT_MOUNT_POINT: &str = procfs::DEFAULT_SELINUX_MOUNT_POINT;

impl FS {
    pub fn new_default_fs() -> Result<Self, std::io::Error> {
        Self::new(DEFAULT_MOUNT_POINT)
    }

    pub fn new(mount_point: &str) -> Result<Self, std::io::Error> {
        let fs = procfs::FS::new(mount_point)?;
        Ok(FS { selinux: fs })
    }
}