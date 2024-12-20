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

use log::debug;
use prometheus::{self, core::Collector, core::Desc, proto::MetricFamily, Gauge, Opts};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::Arc;

struct BtrfsCollector {
    descs: Vec<Desc>,
    logger: slog::Logger,
}

impl BtrfsCollector {
    fn new(logger: slog::Logger) -> Result<Self, Box<dyn std::error::Error>> {
        let descs = vec![
            Desc::new(
                "btrfs_info".to_string(),
                "Filesystem information".to_string(),
                vec!["label".to_string()],
                HashMap::new(),
            )?,
            Desc::new(
                "btrfs_global_rsv_size_bytes".to_string(),
                "Size of global reserve.".to_string(),
                vec![],
                HashMap::new(),
            )?,
            Desc::new(
                "btrfs_commits_total".to_string(),
                "The total number of commits that have occurred.".to_string(),
                vec![],
                HashMap::new(),
            )?,
            Desc::new(
                "btrfs_last_commit_seconds".to_string(),
                "Duration of the most recent commit, in seconds.".to_string(),
                vec![],
                HashMap::new(),
            )?,
            Desc::new(
                "btrfs_max_commit_seconds".to_string(),
                "Duration of the slowest commit, in seconds.".to_string(),
                vec![],
                HashMap::new(),
            )?,
            Desc::new(
                "btrfs_commit_seconds_total".to_string(),
                "Sum of the duration of all commits, in seconds.".to_string(),
                vec![],
                HashMap::new(),
            )?,
        ];

        Ok(BtrfsCollector { descs, logger })
    }

    fn get_metrics(&self, s: &btrfs::Stats, ioctl_stats: Option<&BtrfsIoctlFsStats>) -> Vec<BtrfsMetric> {
        let mut metrics = vec![
            BtrfsMetric {
                name: "info".to_string(),
                desc: "Filesystem information".to_string(),
                value: 1.0,
                metric_type: prometheus::GaugeValue,
                extra_label: vec!["label".to_string()],
                extra_label_value: vec![s.label.clone()],
            },
            BtrfsMetric {
                name: "global_rsv_size_bytes".to_string(),
                desc: "Size of global reserve.".to_string(),
                value: s.allocation.global_rsv_size as f64,
                metric_type: prometheus::GaugeValue,
                extra_label: vec![],
                extra_label_value: vec![],
            },
            BtrfsMetric {
                name: "commits_total".to_string(),
                desc: "The total number of commits that have occurred.".to_string(),
                value: s.commit_stats.commits as f64,
                metric_type: prometheus::CounterValue,
                extra_label: vec![],
                extra_label_value: vec![],
            },
            BtrfsMetric {
                name: "last_commit_seconds".to_string(),
                desc: "Duration of the most recent commit, in seconds.".to_string(),
                value: s.commit_stats.last_commit_ms as f64 / 1000.0,
                metric_type: prometheus::GaugeValue,
                extra_label: vec![],
                extra_label_value: vec![],
            },
            BtrfsMetric {
                name: "max_commit_seconds".to_string(),
                desc: "Duration of the slowest commit, in seconds.".to_string(),
                value: s.commit_stats.max_commit_ms as f64 / 1000.0,
                metric_type: prometheus::GaugeValue,
                extra_label: vec![],
                extra_label_value: vec![],
            },
            BtrfsMetric {
                name: "commit_seconds_total".to_string(),
                desc: "Sum of the duration of all commits, in seconds.".to_string(),
                value: s.commit_stats.total_commit_ms as f64 / 1000.0,
                metric_type: prometheus::CounterValue,
                extra_label: vec![],
                extra_label_value: vec![],
            },
        ];

        metrics.extend(self.get_allocation_stats("data", &s.allocation.data));
        metrics.extend(self.get_allocation_stats("metadata", &s.allocation.metadata));
        metrics.extend(self.get_allocation_stats("system", &s.allocation.system));

        if let Some(ioctl_stats) = ioctl_stats {
            for dev in &ioctl_stats.devices {
                let device = Path::new(&dev.path).file_name().unwrap().to_str().unwrap();
                let extra_labels = vec!["device".to_string(), "btrfs_dev_uuid".to_string()];
                let extra_label_values = vec![device.to_string(), dev.uuid.clone()];

                metrics.push(BtrfsMetric {
                    name: "device_size_bytes".to_string(),
                    desc: "Size of a device that is part of the filesystem.".to_string(),
                    value: dev.total_bytes as f64,
                    metric_type: prometheus::GaugeValue,
                    extra_label: extra_labels.clone(),
                    extra_label_value: extra_label_values.clone(),
                });

                metrics.push(BtrfsMetric {
                    name: "device_unused_bytes".to_string(),
                    desc: "Unused bytes on a device that is part of the filesystem.".to_string(),
                    value: (dev.total_bytes - dev.bytes_used) as f64,
                    metric_type: prometheus::GaugeValue,
                    extra_label: extra_labels.clone(),
                    extra_label_value: extra_label_values.clone(),
                });

                let error_labels = vec!["type".to_string(), "device".to_string(), "btrfs_dev_uuid".to_string()];
                let error_values = vec![
                    dev.write_errs,
                    dev.read_errs,
                    dev.flush_errs,
                    dev.corruption_errs,
                    dev.generation_errs,
                ];
                let error_types = vec!["write", "read", "flush", "corruption", "generation"];

                for (i, error_type) in error_types.iter().enumerate() {
                    metrics.push(BtrfsMetric {
                        name: "device_errors_total".to_string(),
                        desc: "Errors reported for the device".to_string(),
                        value: error_values[i] as f64,
                        metric_type: prometheus::CounterValue,
                        extra_label: error_labels.clone(),
                        extra_label_value: vec![error_type.to_string(), device.to_string(), dev.uuid.clone()],
                    });
                }
            }
        } else {
            for (n, dev) in &s.devices {
                metrics.push(BtrfsMetric {
                    name: "device_size_bytes".to_string(),
                    desc: "Size of a device that is part of the filesystem.".to_string(),
                    value: dev.size as f64,
                    metric_type: prometheus::GaugeValue,
                    extra_label: vec!["device".to_string()],
                    extra_label_value: vec![n.clone()],
                });
            }
        }

        metrics
    }

    fn get_allocation_stats(&self, a: &str, s: &btrfs::AllocationStats) -> Vec<BtrfsMetric> {
        let mut metrics = vec![BtrfsMetric {
            name: "reserved_bytes".to_string(),
            desc: "Amount of space reserved for a data type".to_string(),
            value: s.reserved_bytes as f64,
            metric_type: prometheus::GaugeValue,
            extra_label: vec!["block_group_type".to_string()],
            extra_label_value: vec![a.to_string()],
        }];

        for (layout, stats) in &s.layouts {
            metrics.extend(self.get_layout_stats(a, layout, stats));
        }

        metrics
    }

    fn get_layout_stats(&self, a: &str, l: &str, s: &btrfs::LayoutUsage) -> Vec<BtrfsMetric> {
        vec![
            BtrfsMetric {
                name: "used_bytes".to_string(),
                desc: "Amount of used space by a layout/data type".to_string(),
                value: s.used_bytes as f64,
                metric_type: prometheus::GaugeValue,
                extra_label: vec!["block_group_type".to_string(), "mode".to_string()],
                extra_label_value: vec![a.to_string(), l.to_string()],
            },
            BtrfsMetric {
                name: "size_bytes".to_string(),
                desc: "Amount of space allocated for a layout/data type".to_string(),
                value: s.total_bytes as f64,
                metric_type: prometheus::GaugeValue,
                extra_label: vec!["block_group_type".to_string(), "mode".to_string()],
                extra_label_value: vec![a.to_string(), l.to_string()],
            },
            BtrfsMetric {
                name: "allocation_ratio".to_string(),
                desc: "Data allocation ratio for a layout/data type".to_string(),
                value: s.ratio,
                metric_type: prometheus::GaugeValue,
                extra_label: vec!["block_group_type".to_string(), "mode".to_string()],
                extra_label_value: vec![a.to_string(), l.to_string()],
            },
        ]
    }
}

impl Collector for BtrfsCollector {
    fn desc(&self) -> Vec<&Desc> {
        self.descs.iter().collect()
    }

    fn collect(&self) -> Vec<MetricFamily> {
        let mut metrics = Vec::new();
        let statusfile = Path::new("/sys/fs/btrfs");
        let btrfs_stats = match btrfs::FS::new(statusfile) {
            Ok(fs) => fs.stats(),
            Err(e) => {
                debug!("Failed to read btrfs stats: {:?}", e);
                return metrics;
            }
        };

        for s in btrfs_stats {
            let ioctl_stats = self.get_ioctl_stats(&s.uuid);
            let metrics = self.get_metrics(&s, ioctl_stats.as_ref());
            for m in metrics {
                let desc = prometheus::Desc::new(
                    prometheus::BuildFQName("btrfs", "", &m.name),
                    &m.desc,
                    &m.extra_label,
                    HashMap::new(),
                )
                .unwrap();

                let metric = prometheus::Gauge::new(&m.name, &m.desc)
                    .unwrap()
                    .with_label_values(&m.extra_label_value)
                    .set(m.value);

                metrics.push(metric.collect()[0].clone());
            }
        }

        metrics
    }
}

struct BtrfsMetric {
    name: String,
    desc: String,
    value: f64,
    metric_type: prometheus::proto::MetricType,
    extra_label: Vec<String>,
    extra_label_value: Vec<String>,
}

struct BtrfsIoctlFsStats {
    uuid: String,
    devices: Vec<BtrfsIoctlFsDevStats>,
}

struct BtrfsIoctlFsDevStats {
    path: String,
    uuid: String,
    bytes_used: u64,
    total_bytes: u64,
    write_errs: u64,
    read_errs: u64,
    flush_errs: u64,
    corruption_errs: u64,
    generation_errs: u64,
}