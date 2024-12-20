use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use procfs::ProcFs;
use std::sync::Arc;

struct EntropyCollector {
    fs: ProcFs,
    entropy_avail: Desc,
    entropy_pool_size: Desc,
    logger: Logger,
}

impl EntropyCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        let fs = ProcFs::new("/proc")?;

        Ok(Self {
            fs,
            entropy_avail: Desc::new(
                prometheus::core::build_fq_name("namespace", "", "entropy_available_bits"),
                "Bits of available entropy.",
                vec![],
                None,
            )?,
            entropy_pool_size: Desc::new(
                prometheus::core::build_fq_name("namespace", "", "entropy_pool_size_bits"),
                "Bits of entropy pool.",
                vec![],
                None,
            )?,
            logger,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let stats = self.fs.kernel_random()?;

        if let Some(entropy_available) = stats.entropy_available {
            ch(prometheus::core::MetricFamily::new(
                self.entropy_avail.clone(),
                prometheus::proto::MetricType::GAUGE,
                entropy_available as f64,
                vec![],
            ));
        } else {
            return Err("couldn't get entropy_avail".into());
        }

        if let Some(pool_size) = stats.pool_size {
            ch(prometheus::core::MetricFamily::new(
                self.entropy_pool_size.clone(),
                prometheus::proto::MetricType::GAUGE,
                pool_size as f64,
                vec![],
            ));
        } else {
            return Err("couldn't get entropy poolsize".into());
        }

        Ok(())
    }
}

impl Collector for EntropyCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&self.entropy_avail);
        descs(&self.entropy_pool_size);
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        let mut ch = |metric: MetricFamily| {
            metrics(Box::new(metric));
        };
        if let Err(e) = self.update(&mut ch) {
            self.logger.error("failed to collect entropy metrics", o!("error" => e.to_string()));
        }
    }
}