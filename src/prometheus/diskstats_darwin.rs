use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::sync::Arc;
use iostat::DriveStats;

const DISKSTATS_DEFAULT_IGNORED_DEVICES: &str = "";

struct TypedDescFunc {
    desc: TypedDesc,
    value: fn(&DriveStats) -> f64,
}

struct DiskstatsCollector {
    descs: Vec<TypedDescFunc>,
    device_filter: DeviceFilter,
    logger: Logger,
}

impl DiskstatsCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        let disk_label_names = vec!["device".to_string()];
        let device_filter = new_diskstats_device_filter(&logger)?;

        Ok(Self {
            descs: vec![
                TypedDescFunc {
                    desc: TypedDesc::new("diskstats_reads_completed_total", "The total number of reads completed successfully.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                    value: |stat| stat.num_read as f64,
                },
                TypedDescFunc {
                    desc: TypedDesc::new("diskstats_read_sectors_total", "The total number of sectors read successfully.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                    value: |stat| stat.num_read as f64 / stat.block_size as f64,
                },
                TypedDescFunc {
                    desc: TypedDesc::new("diskstats_read_time_seconds_total", "The total number of seconds spent by all reads.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                    value: |stat| stat.total_read_time.as_secs_f64(),
                },
                TypedDescFunc {
                    desc: TypedDesc::new("diskstats_writes_completed_total", "The total number of writes completed successfully.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                    value: |stat| stat.num_write as f64,
                },
                TypedDescFunc {
                    desc: TypedDesc::new("diskstats_written_sectors_total", "The total number of sectors written successfully.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                    value: |stat| stat.num_write as f64 / stat.block_size as f64,
                },
                TypedDescFunc {
                    desc: TypedDesc::new("diskstats_write_time_seconds_total", "The total number of seconds spent by all writes.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                    value: |stat| stat.total_write_time.as_secs_f64(),
                },
                TypedDescFunc {
                    desc: TypedDesc::new("diskstats_read_bytes_total", "The total number of bytes read successfully.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                    value: |stat| stat.bytes_read as f64,
                },
                TypedDescFunc {
                    desc: TypedDesc::new("diskstats_written_bytes_total", "The total number of bytes written successfully.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                    value: |stat| stat.bytes_written as f64,
                },
                TypedDescFunc {
                    desc: TypedDesc::new("diskstats_read_errors_total", "The total number of read errors.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                    value: |stat| stat.read_errors as f64,
                },
                TypedDescFunc {
                    desc: TypedDesc::new("diskstats_write_errors_total", "The total number of write errors.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                    value: |stat| stat.write_errors as f64,
                },
                TypedDescFunc {
                    desc: TypedDesc::new("diskstats_read_retries_total", "The total number of read retries.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                    value: |stat| stat.read_retries as f64,
                },
                TypedDescFunc {
                    desc: TypedDesc::new("diskstats_write_retries_total", "The total number of write retries.", disk_label_names, prometheus::proto::MetricType::COUNTER),
                    value: |stat| stat.write_retries as f64,
                },
            ],
            device_filter,
            logger,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let disk_stats = DriveStats::collect()?;

        for stats in disk_stats {
            if self.device_filter.ignored(&stats.name) {
                continue;
            }
            for desc in &self.descs {
                let value = (desc.value)(&stats);
                ch(desc.desc.new_metric(value, vec![stats.name.clone()]));
            }
        }
        Ok(())
    }
}

impl Collector for DiskstatsCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        for desc in &self.descs {
            descs(&desc.desc.desc);
        }
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        let mut ch = |metric: MetricFamily| {
            metrics(Box::new(metric));
        };
        if let Err(e) = self.update(&mut ch) {
            self.logger.error("failed to collect disk stats", o!("error" => e.to_string()));
        }
    }
}