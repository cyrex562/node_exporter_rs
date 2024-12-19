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
use std::io;
use std::path::Path;

#[derive(Debug)]
pub struct ClockSource {
    name: String,
    available: Vec<String>,
    current: String,
}

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn clock_sources(&self) -> Result<Vec<ClockSource>, io::Error> {
        let pattern = format!("{}/devices/system/clocksource/clocksource[0-9]*", self.sys_path);
        let clocksource_paths = glob::glob(&pattern)?;

        let mut clocksources = Vec::new();
        for clocksource_path in clocksource_paths {
            let clocksource_path = clocksource_path?;
            let clocksource_name = clocksource_path.file_name().unwrap().to_str().unwrap().trim_start_matches("clocksource").to_string();

            let mut clocksource = parse_clocksource(&clocksource_path)?;
            clocksource.name = clocksource_name;
            clocksources.push(clocksource);
        }

        Ok(clocksources)
    }
}

fn parse_clocksource(clocksource_path: &Path) -> Result<ClockSource, io::Error> {
    let available = fs::read_to_string(clocksource_path.join("available_clocksource"))?
        .trim()
        .split_whitespace()
        .map(String::from)
        .collect();

    let current = fs::read_to_string(clocksource_path.join("current_clocksource"))?.trim().to_string();

    Ok(ClockSource {
        name: String::new(),
        available,
        current,
    })
}