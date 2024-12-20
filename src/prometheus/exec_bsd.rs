use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::sync::Arc;

struct BsdSysctl {
    name: &'static str,
    description: &'static str,
    mib: &'static str,
}

impl BsdSysctl {
    fn value(&self) -> Result<f64, Box<dyn std::error::Error>> {
        // Implement the logic to fetch the value from sysctl
        Ok(0.0) // Placeholder
    }
}

struct ExecCollector {
    sysctls: Vec<BsdSysctl>,
    logger: Logger,
}

impl ExecCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            sysctls: vec![
                BsdSysctl {
                    name: "exec_context_switches_total",
                    description: "Context switches since system boot. Resets at architecture unsigned integer.",
                    mib: "vm.stats.sys.v_swtch",
                },
                BsdSysctl {
                    name: "exec_traps_total",
                    description: "Traps since system boot. Resets at architecture unsigned integer.",
                    mib: "vm.stats.sys.v_trap",
                },
                BsdSysctl {
                    name: "exec_system_calls_total",
                    description: "System calls since system boot. Resets at architecture unsigned integer.",
                    mib: "vm.stats.sys.v_syscall",
                },
                BsdSysctl {
                    name: "exec_device_interrupts_total",
                    description: "Device interrupts since system boot. Resets at architecture unsigned integer.",
                    mib: "vm.stats.sys.v_intr",
                },
                BsdSysctl {
                    name: "exec_software_interrupts_total",
                    description: "Software interrupts since system boot. Resets at architecture unsigned integer.",
                    mib: "vm.stats.sys.v_soft",
                },
                BsdSysctl {
                    name: "exec_forks_total",
                    description: "Number of fork() calls since system boot. Resets at architecture unsigned integer.",
                    mib: "vm.stats.vm.v_forks",
                },
            ],
            logger,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        for m in &self.sysctls {
            let v = m.value()?;
            ch(prometheus::core::MetricFamily::new(
                Desc::new(
                    prometheus::core::build_fq_name("namespace", "", m.name),
                    m.description,
                    vec![],
                    None,
                )?,
                prometheus::proto::MetricType::COUNTER,
                v,
                vec![],
            ));
        }
        Ok(())
    }
}

impl Collector for ExecCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        for m in &self.sysctls {
            descs(&Desc::new(
                prometheus::core::build_fq_name("namespace", "", m.name),
                m.description,
                vec![],
                None,
            ).unwrap());
        }
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        let mut ch = |metric: MetricFamily| {
            metrics(Box::new(metric));
        };
        if let Err(e) = self.update(&mut ch) {
            self.logger.error("failed to collect exec metrics", o!("error" => e.to_string()));
        }
    }
}