// Copyright 2015 The Prometheus Authors
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
use std::sync::Arc;
use log::debug;
use prometheus::{self, core::Collector, core::Desc, proto::MetricFamily, Gauge, Opts};

struct BondingCollector {
    slaves: Desc,
    active: Desc,
    logger: slog::Logger,
}

impl BondingCollector {
    fn new(logger: slog::Logger) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(BondingCollector {
            slaves: Desc::new(
                "bonding_slaves".to_string(),
                "Number of configured slaves per bonding interface.".to_string(),
                vec!["master".to_string()],
                HashMap::new(),
            )?,
            active: Desc::new(
                "bonding_active".to_string(),
                "Number of active slaves per bonding interface.".to_string(),
                vec!["master".to_string()],
                HashMap::new(),
            )?,
            logger,
        })
    }

    fn read_bonding_stats(root: &Path) -> Result<HashMap<String, [i32; 2]>, io::Error> {
        let mut status = HashMap::new();
        let masters = fs::read_to_string(root.join("bonding_masters"))?;
        for master in masters.split_whitespace() {
            let slaves = fs::read_to_string(root.join(master).join("bonding").join("slaves"))?;
            let mut sstat = [0, 0];
            for slave in slaves.split_whitespace() {
                let state_path = root.join(master).join(format!("lower_{}", slave)).join("bonding_slave").join("mii_status");
                let state = match fs::read_to_string(&state_path) {
                    Ok(state) => state,
                    Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                        let state_path = root.join(master).join(format!("slave_{}", slave)).join("bonding_slave").join("mii_status");
                        fs::read_to_string(state_path)?
                    }
                    Err(e) => return Err(e),
                };
                sstat[0] += 1;
                if state.trim() == "up" {
                    sstat[1] += 1;
                }
            }
            status.insert(master.to_string(), sstat);
        }
        Ok(status)
    }
}

impl Collector for BondingCollector {
    fn desc(&self) -> Vec<&Desc> {
        vec![&self.slaves, &self.active]
    }

    fn collect(&self) -> Vec<MetricFamily> {
        let mut metrics = Vec::new();
        let statusfile = Path::new("/sys/class/net");
        let bonding_stats = match Self::read_bonding_stats(statusfile) {
            Ok(stats) => stats,
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    debug!("Not collecting bonding, file does not exist: {:?}", statusfile);
                    return metrics;
                }
                panic!("Failed to read bonding stats: {:?}", e);
            }
        };

        for (master, status) in bonding_stats {
            metrics.push(prometheus::Gauge::new("bonding_slaves", "Number of configured slaves per bonding interface.")
                .unwrap()
                .with_label_values(&[&master])
                .set(status[0] as f64));
            metrics.push(prometheus::Gauge::new("bonding_active", "Number of active slaves per bonding interface.")
                .unwrap()
                .with_label_values(&[&master])
                .set(status[1] as f64));
        }

        metrics
    }
}