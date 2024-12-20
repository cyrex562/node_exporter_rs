use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::sync::Arc;

const CGROUPS_COLLECTOR_SUBSYSTEM: &str = "cgroups";

struct CgroupSummaryCollector {
    fs: procfs::ProcFs,
    cgroups: Arc<Desc>,
    enabled: Arc<Desc>,
    logger: Logger,
}

impl CgroupSummaryCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        let fs = procfs::ProcFs::new()?;
        let cgroups = Desc::new(
            prometheus::core::build_fq_name("namespace", CGROUPS_COLLECTOR_SUBSYSTEM, "cgroups"),
            "Current cgroup number of the subsystem.",
            vec!["subsys_name".to_string()],
            None,
        )?;
        let enabled = Desc::new(
            prometheus::core::build_fq_name("namespace", CGROUPS_COLLECTOR_SUBSYSTEM, "enabled"),
            "Current cgroup number of the subsystem.",
            vec!["subsys_name".to_string()],
            None,
        )?;
        Ok(Self {
            fs,
            cgroups: Arc::new(cgroups),
            enabled: Arc::new(enabled),
            logger,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let cgroup_summaries = self.fs.cgroup_summaries()?;
        for cs in cgroup_summaries {
            ch(prometheus::core::MetricFamily::new(
                self.cgroups.clone(),
                prometheus::proto::MetricType::GAUGE,
                cs.cgroups as f64,
                vec![cs.subsys_name.clone()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.enabled.clone(),
                prometheus::proto::MetricType::GAUGE,
                cs.enabled as f64,
                vec![cs.subsys_name.clone()],
            ));
        }
        Ok(())
    }
}