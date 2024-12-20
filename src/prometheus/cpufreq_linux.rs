use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::sync::Arc;
use sysfs::SysFs;

struct CpuFreqCollector {
    fs: SysFs,
    logger: Logger,
}

impl CpuFreqCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        let fs = SysFs::new("/sys")?;
        Ok(Self { fs, logger })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let cpu_freqs = self.fs.system_cpufreq()?;

        for stats in cpu_freqs {
            if let Some(freq) = stats.cpuinfo_current_frequency {
                ch(prometheus::core::MetricFamily::new(
                    CPU_FREQ_HERTZ_DESC.clone(),
                    prometheus::proto::MetricType::GAUGE,
                    freq as f64 * 1000.0,
                    vec![stats.name.clone()],
                ));
            }
            if let Some(freq) = stats.cpuinfo_minimum_frequency {
                ch(prometheus::core::MetricFamily::new(
                    CPU_FREQ_MIN_DESC.clone(),
                    prometheus::proto::MetricType::GAUGE,
                    freq as f64 * 1000.0,
                    vec![stats.name.clone()],
                ));
            }
            if let Some(freq) = stats.cpuinfo_maximum_frequency {
                ch(prometheus::core::MetricFamily::new(
                    CPU_FREQ_MAX_DESC.clone(),
                    prometheus::proto::MetricType::GAUGE,
                    freq as f64 * 1000.0,
                    vec![stats.name.clone()],
                ));
            }
            if let Some(freq) = stats.scaling_current_frequency {
                ch(prometheus::core::MetricFamily::new(
                    CPU_FREQ_SCALING_FREQ_DESC.clone(),
                    prometheus::proto::MetricType::GAUGE,
                    freq as f64 * 1000.0,
                    vec![stats.name.clone()],
                ));
            }
            if let Some(freq) = stats.scaling_minimum_frequency {
                ch(prometheus::core::MetricFamily::new(
                    CPU_FREQ_SCALING_FREQ_MIN_DESC.clone(),
                    prometheus::proto::MetricType::GAUGE,
                    freq as f64 * 1000.0,
                    vec![stats.name.clone()],
                ));
            }
            if let Some(freq) = stats.scaling_maximum_frequency {
                ch(prometheus::core::MetricFamily::new(
                    CPU_FREQ_SCALING_FREQ_MAX_DESC.clone(),
                    prometheus::proto::MetricType::GAUGE,
                    freq as f64 * 1000.0,
                    vec![stats.name.clone()],
                ));
            }
            if !stats.governor.is_empty() {
                for g in stats.available_governors.split_whitespace() {
                    let state = if g == stats.governor { 1.0 } else { 0.0 };
                    ch(prometheus::core::MetricFamily::new(
                        CPU_FREQ_SCALING_GOVERNOR_DESC.clone(),
                        prometheus::proto::MetricType::GAUGE,
                        state,
                        vec![stats.name.clone(), g.to_string()],
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Collector for CpuFreqCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&CPU_FREQ_HERTZ_DESC);
        descs(&CPU_FREQ_MIN_DESC);
        descs(&CPU_FREQ_MAX_DESC);
        descs(&CPU_FREQ_SCALING_FREQ_DESC);
        descs(&CPU_FREQ_SCALING_FREQ_MIN_DESC);
        descs(&CPU_FREQ_SCALING_FREQ_MAX_DESC);
        descs(&CPU_FREQ_SCALING_GOVERNOR_DESC);
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