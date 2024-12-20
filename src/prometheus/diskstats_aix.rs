use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::sync::Arc;
use perfstat::DiskStat;

const DISKSTATS_DEFAULT_IGNORED_DEVICES: &str = "";

struct DiskstatsCollector {
    rbytes: TypedDesc,
    wbytes: TypedDesc,
    time: TypedDesc,
    device_filter: DeviceFilter,
    logger: Logger,
    tick_per_second: i64,
}

impl DiskstatsCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        let ticks = tick_per_second()?;
        let device_filter = new_diskstats_device_filter(&logger)?;

        Ok(Self {
            rbytes: TypedDesc::new("diskstats_read_bytes_total", "Total number of bytes read.", vec!["device"], prometheus::proto::MetricType::COUNTER),
            wbytes: TypedDesc::new("diskstats_written_bytes_total", "Total number of bytes written.", vec!["device"], prometheus::proto::MetricType::COUNTER),
            time: TypedDesc::new("diskstats_io_time_seconds_total", "Total I/O time in seconds.", vec!["device"], prometheus::proto::MetricType::COUNTER),
            device_filter,
            logger,
            tick_per_second: ticks,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let stats = DiskStat::collect()?;

        for stat in stats {
            if self.device_filter.ignored(&stat.name) {
                continue;
            }
            ch(self.rbytes.new_metric((stat.rblks * 512) as f64, vec![stat.name.clone()]));
            ch(self.wbytes.new_metric((stat.wblks * 512) as f64, vec![stat.name.clone()]));
            ch(self.time.new_metric((stat.time / self.tick_per_second) as f64, vec![stat.name]));
        }
        Ok(())
    }
}

impl Collector for DiskstatsCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&self.rbytes.desc);
        descs(&self.wbytes.desc);
        descs(&self.time.desc);
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