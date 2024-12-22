use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use glob::glob;

pub struct Stats {
    // The name of the bcache used to source these statistics.
    pub name: String,
    pub bcache: BcacheStats,
    pub bdevs: Vec<BdevStats>,
    pub caches: Vec<CacheStats>,
}

// BcacheStats contains statistics tied to a bcache ID.
pub struct BcacheStats {
    pub average_key_size: u64,
    pub btree_cache_size: u64,
    pub cache_available_percent: u64,
    pub congested: u64,
    pub root_usage_percent: u64,
}

// BdevStats contains statistics for a bcache backing device.
pub struct BdevStats {
    pub name: String,
    pub dirty_data: u64,
    pub io_errors: u64,
    pub metadata_written: u64,
    pub written: u64,
}

// CacheStats contains statistics for a bcache cache device.
pub struct CacheStats {
    pub name: String,
    pub io_errors: u64,
    pub metadata_written: u64,
    pub written: u64,
}



pub struct FS {
    sys: Arc<Mutex<PathBuf>>,
}

impl FS {
    pub fn new_default_fs() -> io::Result<Self> {
        Self::new(fs::default_sys_mount_point())
    }

    pub fn new(mount_point: String) -> io::Result<Self> {
        let mount_point = if mount_point.trim().is_empty() {
            fs::default_sys_mount_point()
        } else {
            PathBuf::from(mount_point)
        };

        if mount_point.exists() {
            Ok(FS {
                sys: Arc::new(Mutex::new(mount_point)),
            })
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Mount point can't be read",
            ))
        }
    }

    pub fn stats(&self) -> io::Result<Vec<Stats>> {
        self.stats_internal(true)
    }

    pub fn stats_without_priority(&self) -> io::Result<Vec<Stats>> {
        self.stats_internal(false)
    }

    fn stats_internal(&self, priority_stats: bool) -> io::Result<Vec<Stats>> {
        let sys_path = self.sys.lock().unwrap();
        let pattern = sys_path.join("fs/bcache/*-*").to_string_lossy().to_string();
        let matches = glob(&pattern).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut stats = Vec::new();
        for entry in matches {
            match entry {
                Ok(uuid_path) => {
                    let name = uuid_path.file_name().unwrap().to_string_lossy().to_string();
                    match get_stats(&uuid_path, priority_stats) {
                        Ok(s) => stats.push(s),
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
            }
        }

        Ok(stats)
    }
}

mod fs {
    use std::path::PathBuf;

    pub fn default_sys_mount_point() -> PathBuf {
        PathBuf::from("/sys")
    }
}

pub struct Stats {
    // Define the fields for Stats struct
}

fn get_stats(path: &Path, priority_stats: bool) -> io::Result<Stats> {
    // Implement the logic to get stats from the given path
    Ok(Stats {
        // Initialize the fields
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_fs_bcache_stats() {
        let bcache = FS::new("testdata/fixtures/sys".to_string()).expect("failed to access bcache fs");
        let stats = bcache.stats().expect("failed to parse bcache stats");

        let tests = vec![
            TestStats {
                name: "deaddd54-c735-46d5-868e-f331c5fd7c74".to_string(),
                bdevs: 1,
                caches: 1,
            },
        ];

        for test in tests {
            let stat = stats.iter().find(|s| s.name == test.name).expect("stat not found");
            assert_eq!(stat.bdevs.len(), test.bdevs);
            assert_eq!(stat.caches.len(), test.caches);
        }
    }

    struct TestStats {
        name: String,
        bdevs: usize,
        caches: usize,
    }
}

// Copyright 2017 The Prometheus Authors
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

use prometheus::{self, Counter, Gauge, Opts, Registry};
use slog::Logger;
use std::sync::Arc;

lazy_static! {
    static ref PRIORITY_STATS: bool = {
        let matches = clap::App::new("collector")
            .arg(clap::Arg::with_name("collector.bcache.priorityStats")
                .long("collector.bcache.priorityStats")
                .help("Expose expensive priority stats.")
                .takes_value(false))
            .get_matches();
        matches.is_present("collector.bcache.priorityStats")
    };
}

pub struct BcacheCollector {
    fs: Arc<bcache::FS>,
    logger: Logger,
}

impl BcacheCollector {
    pub fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        let fs = bcache::FS::new("/sys/fs/bcache")?;
        Ok(BcacheCollector {
            fs: Arc::new(fs),
            logger,
        })
    }

    pub fn update(&self, registry: &Registry) -> Result<(), Box<dyn std::error::Error>> {
        let stats = if *PRIORITY_STATS {
            self.fs.stats()?
        } else {
            self.fs.stats_without_priority()?
        };

        for stat in stats {
            self.update_bcache_stats(registry, &stat)?;
        }
        Ok(())
    }

    fn update_bcache_stats(
        &self,
        registry: &Registry,
        stat: &bcache::Stats,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let subsystem = "bcache";
        let dev_label = vec!["uuid"];

        let metrics = vec![
            ("average_key_size_sectors", "Average data per key in the btree (sectors).", stat.bcache.average_key_size as f64, Gauge::new),
            ("btree_cache_size_bytes", "Amount of memory currently used by the btree cache.", stat.bcache.btree_cache_size as f64, Gauge::new),
            ("cache_available_percent", "Percentage of cache device without dirty data, usable for writeback (may contain clean cached data).", stat.bcache.cache_available_percent as f64, Gauge::new),
            ("congested", "Congestion.", stat.bcache.congested as f64, Gauge::new),
            ("root_usage_percent", "Percentage of the root btree node in use (tree depth increases if too high).", stat.bcache.root_usage_percent as f64, Gauge::new),
            ("tree_depth", "Depth of the btree.", stat.bcache.tree_depth as f64, Gauge::new),
            ("active_journal_entries", "Number of journal entries that are newer than the index.", stat.bcache.internal.active_journal_entries as f64, Gauge::new),
            ("btree_nodes", "Total nodes in the btree.", stat.bcache.internal.btree_nodes as f64, Gauge::new),
            ("btree_read_average_duration_seconds", "Average btree read duration.", stat.bcache.internal.btree_read_average_duration_nanoseconds as f64 * 1e-9, Gauge::new),
            ("cache_read_races_total", "Counts instances where while data was being read from the cache, the bucket was reused and invalidated - i.e. where the pointer was stale after the read completed.", stat.bcache.internal.cache_read_races as f64, Counter::new),
        ];

        for (name, desc, value, metric_type) in metrics {
            let opts = Opts::new(name, desc).namespace(subsystem).subsystem(subsystem);
            let metric = metric_type(opts)?;
            metric.set(value);
            registry.register(Box::new(metric))?;
        }

        for bdev in &stat.bdevs {
            let metrics = vec![
                ("dirty_data_bytes", "Amount of dirty data for this backing device in the cache.", bdev.dirty_data as f64, Gauge::new),
                ("dirty_target_bytes", "Current dirty data target threshold for this backing device in bytes.", bdev.writeback_rate_debug.target as f64, Gauge::new),
                ("writeback_rate", "Current writeback rate for this backing device in bytes.", bdev.writeback_rate_debug.rate as f64, Gauge::new),
                ("writeback_rate_proportional_term", "Current result of proportional controller, part of writeback rate", bdev.writeback_rate_debug.proportional as f64, Gauge::new),
                ("writeback_rate_integral_term", "Current result of integral controller, part of writeback rate", bdev.writeback_rate_debug.integral as f64, Gauge::new),
                ("writeback_change", "Last writeback rate change step for this backing device.", bdev.writeback_rate_debug.change as f64, Gauge::new),
            ];

            for (name, desc, value, metric_type) in metrics {
                let opts = Opts::new(name, desc).namespace(subsystem).subsystem(subsystem).const_label("backing_device", &bdev.name);
                let metric = metric_type(opts)?;
                metric.set(value);
                registry.register(Box::new(metric))?;
            }

            let period_stats_metrics = self.bcache_period_stats_to_metric(&bdev.total, &bdev.name);
            for (name, desc, value, metric_type) in period_stats_metrics {
                let opts = Opts::new(name, desc).namespace(subsystem).subsystem(subsystem).const_label("backing_device", &bdev.name);
                let metric = metric_type(opts)?;
                metric.set(value);
                registry.register(Box::new(metric))?;
            }
        }

        for cache in &stat.caches {
            let metrics = vec![
                ("io_errors", "Number of errors that have occurred, decayed by io_error_halflife.", cache.io_errors as f64, Gauge::new),
                ("metadata_written_bytes_total", "Sum of all non data writes (btree writes and all other metadata).", cache.metadata_written as f64, Counter::new),
                ("written_bytes_total", "Sum of all data that has been written to the cache.", cache.written as f64, Counter::new),
            ];

            for (name, desc, value, metric_type) in metrics {
                let opts = Opts::new(name, desc).namespace(subsystem).subsystem(subsystem).const_label("cache_device", &cache.name);
                let metric = metric_type(opts)?;
                metric.set(value);
                registry.register(Box::new(metric))?;
            }

            if *PRIORITY_STATS {
                let priority_stats_metrics = vec![
                    ("priority_stats_unused_percent", "The percentage of the cache that doesn't contain any data.", cache.priority.unused_percent as f64, Gauge::new),
                    ("priority_stats_metadata_percent", "Bcache's metadata overhead.", cache.priority.metadata_percent as f64, Gauge::new),
                ];

                for (name, desc, value, metric_type) in priority_stats_metrics {
                    let opts = Opts::new(name, desc).namespace(subsystem).subsystem(subsystem).const_label("cache_device", &cache.name);
                    let metric = metric_type(opts)?;
                    metric.set(value);
                    registry.register(Box::new(metric))?;
                }
            }
        }

        Ok(())
    }

    fn bcache_period_stats_to_metric(
        &self,
        ps: &bcache::PeriodStats,
        label_value: &str,
    ) -> Vec<(&str, &str, f64, fn(Opts) -> Result<Box<dyn prometheus::core::Collector>, prometheus::Error>)> {
        let label = vec!["backing_device"];

        let mut metrics = vec![
            ("bypassed_bytes_total", "Amount of IO (both reads and writes) that has bypassed the cache.", ps.bypassed as f64, Counter::new),
            ("cache_hits_total", "Hits counted per individual IO as bcache sees them.", ps.cache_hits as f64, Counter::new),
            ("cache_misses_total", "Misses counted per individual IO as bcache sees them.", ps.cache_misses as f64, Counter::new),
            ("cache_bypass_hits_total", "Hits for IO intended to skip the cache.", ps.cache_bypass_hits as f64, Counter::new),
            ("cache_bypass_misses_total", "Misses for IO intended to skip the cache.", ps.cache_bypass_misses as f64, Counter::new),
            ("cache_miss_collisions_total", "Instances where data insertion from cache miss raced with write (data already present).", ps.cache_miss_collisions as f64, Counter::new),
        ];

        if ps.cache_readaheads != 0 {
            metrics.push((
                "cache_readaheads_total",
                "Count of times readahead occurred.",
                ps.cache_readaheads as f64,
                Counter::new,
            ));
        }

        metrics
    }
}