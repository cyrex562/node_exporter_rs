use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::ffi::CString;
use std::ptr;

struct CpuCollector {
    cpu: TypedDesc,
    logger: Logger,
    tick_per_second: i64,
}

impl CpuCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
        if ticks == -1 {
            return Err("failed to get clock ticks per second".into());
        }
        Ok(Self {
            cpu: TypedDesc::new("node_cpu_seconds_total", "Total CPU time", vec!["cpu", "mode"]),
            logger,
            tick_per_second: ticks,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let stats = perfstat::cpu_stat()?;
        for (n, stat) in stats.iter().enumerate() {
            ch(self.cpu.must_new_const_metric(stat.user as f64 / self.tick_per_second as f64, vec![n.to_string(), "user".to_string()]));
            ch(self.cpu.must_new_const_metric(stat.sys as f64 / self.tick_per_second as f64, vec![n.to_string(), "system".to_string()]));
            ch(self.cpu.must_new_const_metric(stat.idle as f64 / self.tick_per_second as f64, vec![n.to_string(), "idle".to_string()]));
            ch(self.cpu.must_new_const_metric(stat.wait as f64 / self.tick_per_second as f64, vec![n.to_string(), "wait".to_string()]));
        }
        Ok(())
    }
}

impl Collector for CpuCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&self.cpu.desc);
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        let mut ch = |metric: MetricFamily| {
            metrics(Box::new(metric));
        };
        if let Err(e) = self.update(&mut ch) {
            self.logger.error("failed to collect CPU metrics", o!("error" => e.to_string()));
        }
    }
}