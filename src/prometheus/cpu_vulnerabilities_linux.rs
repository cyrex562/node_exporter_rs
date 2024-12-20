use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::sync::Arc;
use sysfs::SysFs;

const CPU_VULNERABILITIES_COLLECTOR: &str = "cpu_vulnerabilities";

lazy_static! {
    static ref VULNERABILITY_DESC: Desc = Desc::new(
        prometheus::core::build_fq_name("namespace", CPU_VULNERABILITIES_COLLECTOR, "info"),
        "Details of each CPU vulnerability reported by sysfs. The value of the series is an int encoded state of the vulnerability. The same state is stored as a string in the label",
        vec!["codename".to_string(), "state".to_string(), "mitigation".to_string()],
        None,
    ).unwrap();
}

struct CpuVulnerabilitiesCollector;

impl CpuVulnerabilitiesCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let fs = SysFs::new("/sys")?;
        let vulnerabilities = fs.cpu_vulnerabilities()?;

        for vulnerability in vulnerabilities {
            ch(prometheus::core::MetricFamily::new(
                VULNERABILITY_DESC.clone(),
                prometheus::proto::MetricType::GAUGE,
                1.0,
                vec![
                    vulnerability.codename,
                    sysfs::vulnerability_human_encoding(vulnerability.state),
                    vulnerability.mitigation,
                ],
            ));
        }
        Ok(())
    }
}

impl Collector for CpuVulnerabilitiesCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&VULNERABILITY_DESC);
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        let mut ch = |metric: MetricFamily| {
            metrics(Box::new(metric));
        };
        if let Err(e) = self.update(&mut ch) {
            eprintln!("failed to collect CPU vulnerabilities metrics: {}", e);
        }
    }
}