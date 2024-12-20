use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::ffi::CString;
use std::ptr;
use std::sync::Arc;
use std::time::Duration;
use libc::{c_int, c_void};
use kstat::Kstat;

struct CpuCollector {
    cpu: TypedDesc,
    logger: Logger,
}

impl CpuCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            cpu: TypedDesc::new("node_cpu_seconds_total", "Seconds the CPUs spent in each mode.", vec!["cpu", "mode"]),
            logger,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let ncpus = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };

        let tok = Kstat::open()?;
        for cpu in 0..ncpus {
            let ks_cpu = tok.lookup("cpu", cpu as i32, "sys")?;
            for (k, v) in [("idle", "cpu_nsec_idle"), ("kernel", "cpu_nsec_kernel"), ("user", "cpu_nsec_user"), ("wait", "cpu_nsec_wait")].iter() {
                let kstat_value = ks_cpu.get_named(v)?;
                ch(self.cpu.must_new_const_metric(kstat_value.uint_val() as f64 / 1e9, vec![cpu.to_string(), k.to_string()]));
            }
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