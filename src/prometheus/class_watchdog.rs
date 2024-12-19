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

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

const WATCHDOG_CLASS_PATH: &str = "class/watchdog";

#[derive(Debug, Default)]
pub struct WatchdogStats {
    name: String,
    bootstatus: Option<i64>,
    options: Option<String>,
    fw_version: Option<i64>,
    identity: Option<String>,
    nowayout: Option<i64>,
    state: Option<String>,
    status: Option<String>,
    timeleft: Option<i64>,
    timeout: Option<i64>,
    pretimeout: Option<i64>,
    pretimeout_governor: Option<String>,
    access_cs0: Option<i64>,
}

pub type WatchdogClass = HashMap<String, WatchdogStats>;

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn watchdog_class(&self) -> Result<WatchdogClass, io::Error> {
        let path = Path::new(&self.sys_path).join(WATCHDOG_CLASS_PATH);
        let dirs = fs::read_dir(path)?;

        let mut wds = WatchdogClass::new();
        for dir in dirs {
            let dir = dir?;
            let stats = self.parse_watchdog(&dir.file_name().to_string_lossy())?;
            wds.insert(stats.name.clone(), stats);
        }

        Ok(wds)
    }

    fn parse_watchdog(&self, wd_name: &str) -> Result<WatchdogStats, io::Error> {
        let path = Path::new(&self.sys_path).join(WATCHDOG_CLASS_PATH).join(wd_name);
        let mut wd = WatchdogStats {
            name: wd_name.to_string(),
            ..Default::default()
        };

        for field in &[
            "bootstatus",
            "options",
            "fw_version",
            "identity",
            "nowayout",
            "state",
            "status",
            "timeleft",
            "timeout",
            "pretimeout",
            "pretimeout_governor",
            "access_cs0",
        ] {
            let value = match fs::read_to_string(path.join(field)) {
                Ok(val) => val.trim().to_string(),
                Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e),
            };

            match *field {
                "bootstatus" => wd.bootstatus = value.parse().ok(),
                "options" => wd.options = Some(value),
                "fw_version" => wd.fw_version = value.parse().ok(),
                "identity" => wd.identity = Some(value),
                "nowayout" => wd.nowayout = value.parse().ok(),
                "state" => wd.state = Some(value),
                "status" => wd.status = Some(value),
                "timeleft" => wd.timeleft = value.parse().ok(),
                "timeout" => wd.timeout = value.parse().ok(),
                "pretimeout" => wd.pretimeout = value.parse().ok(),
                "pretimeout_governor" => wd.pretimeout_governor = Some(value),
                "access_cs0" => wd.access_cs0 = value.parse().ok(),
                _ => {}
            }
        }

        Ok(wd)
    }
}