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

use prometheus::{self, core::Collector, core::Desc, proto::MetricFamily, Gauge, Opts};
use std::sync::Arc;
use log::error;
use kstat::KstatCtl;

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

        let kstat_ctl = match KstatCtl::new() {
            Ok(kc) => kc,
            Err(e) => {
                error!("Failed to open kstat: {:?}", e);
                return metrics;
            }
        };

        let kstat = match kstat_ctl.lookup("unix", 0, "system_misc") {
            Ok(ks) => ks,
            Err(e) => {
                error!("Failed to lookup kstat: {:?}", e);
                return metrics;
            }
        };

        let boot_time = match kstat.get_named("boot_time") {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to get boot_time: {:?}", e);
                return metrics;
            }
        };

        let v = boot_time.value().unwrap_or(0) as f64;

        metrics.push(prometheus::Gauge::new("boot_time_seconds", "Unix time of last boot, including microseconds.")
            .unwrap()
            .set(v));

        metrics
    }
}