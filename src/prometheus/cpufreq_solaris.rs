use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::ffi::CString;
use std::ptr;
use std::sync::Arc;
use libc::{c_int, c_void};
use kstat::Kstat;

struct CpuFreqCollector {
    logger: Logger,
}

impl CpuFreqCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { logger })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let ncpus = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };

        let tok = Kstat::open()?;
        for cpu in 0..ncpus {
            let ks_cpu_info = tok.lookup("cpu_info", cpu as i32, &format!("cpu_info{}", cpu))?;
            let cpu_freq_v = ks_cpu_info.get_named("current_clock_Hz")?;
            let cpu_freq_max_v = ks_cpu_info.get_named("clock_MHz")?;

            let lcpu = cpu.to_string();
            ch(prometheus::core::MetricFamily::new(
                CPU_FREQ_HERTZ_DESC.clone(),
                prometheus::proto::MetricType::GAUGE,
                cpu_freq_v.uint_val() as f64,
                vec![lcpu.clone()],
            ));
            ch(prometheus::core::MetricFamily::new(
                CPU_FREQ_MAX_DESC.clone(),
                prometheus::proto::MetricType::GAUGE,
                cpu_freq_max_v.int_val() as f64 * 1e6,
                vec![lcpu],
            ));
        }
        Ok(())
    }
}

impl Collector for CpuFreqCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&CPU_FREQ_HERTZ_DESC);
        descs(&CPU_FREQ_MAX_DESC);
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        let mut ch = |metric: MetricFamily| {
            metrics(Box::new(metric));
        };
        if let Err(e) = self.update(&mut ch) {
            self.logger.error("failed to collect CPU frequency metrics", o!("error" => e.to_string()));
        }
    }
}