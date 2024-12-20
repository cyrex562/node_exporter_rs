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

#[cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "openbsd", target_os = "netbsd", target_os = "macos"))]
use prometheus::{self, core::Collector, core::Desc, proto::MetricFamily, Gauge, Opts};
use std::sync::Arc;
use log::error;
use nix::sys::sysctl::Sysctl;

struct BootTimeCollector {
    desc: Desc,
}

impl BootTimeCollector {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(BootTimeCollector {
            desc: Desc::new(
                "boot_time_seconds".to_string(),
                "Unix time of last boot, including microseconds.".to_string(),
                vec![],
                HashMap::new(),
            )?,
        })
    }
}

impl Collector for BootTimeCollector {
    fn desc(&self) -> Vec<&Desc> {
        vec![&self.desc]
    }

    fn collect(&self) -> Vec<MetricFamily> {
        let mut metrics = Vec::new();

        match nix::sys::sysctl::value("kern.boottime") {
            Ok(nix::sys::sysctl::CtlValue::Struct(tv)) => {
                let v = tv.sec as f64 + (tv.usec as f64 / 1_000_000.0);
                metrics.push(prometheus::Gauge::new("boot_time_seconds", "Unix time of last boot, including microseconds.")
                    .unwrap()
                    .set(v));
            }
            Err(e) => {
                error!("Failed to retrieve boot time: {:?}", e);
            }
            _ => {
                error!("Unexpected sysctl value type for kern.boottime");
            }
        }

        metrics
    }
}