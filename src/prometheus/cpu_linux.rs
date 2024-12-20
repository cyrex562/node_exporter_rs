use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::Path;
use regex::Regex;
use procfs::ProcFs;
use sysfs::SysFs;

const JUMP_BACK_SECONDS: f64 = 3.0;

struct CpuCollector {
    fs: ProcFs,
    cpu: Desc,
    cpu_info: Desc,
    cpu_frequency_hz: Desc,
    cpu_flags_info: Desc,
    cpu_bugs_info: Desc,
    cpu_guest: Desc,
    cpu_core_throttle: Desc,
    cpu_package_throttle: Desc,
    cpu_isolated: Desc,
    logger: Logger,
    cpu_stats: Mutex<HashMap<i64, procfs::CPUStat>>,
    isolated_cpus: Vec<u16>,
    cpu_flags_include_regexp: Option<Regex>,
    cpu_bugs_include_regexp: Option<Regex>,
}

impl CpuCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        let fs = ProcFs::new("/proc")?;
        let sysfs = SysFs::new("/sys")?;
        let isolated_cpus = sysfs.isolated_cpus().unwrap_or_default();

        Ok(Self {
            fs,
            cpu: Desc::new(
                prometheus::core::build_fq_name("namespace", "cpu", "seconds_total"),
                "Seconds the CPUs spent in each mode.",
                vec!["cpu".to_string(), "mode".to_string()],
                None,
            )?,
            cpu_info: Desc::new(
                prometheus::core::build_fq_name("namespace", "cpu", "info"),
                "CPU information from /proc/cpuinfo.",
                vec!["package".to_string(), "core".to_string(), "cpu".to_string(), "vendor".to_string(), "family".to_string(), "model".to_string(), "model_name".to_string(), "microcode".to_string(), "stepping".to_string(), "cachesize".to_string()],
                None,
            )?,
            cpu_frequency_hz: Desc::new(
                prometheus::core::build_fq_name("namespace", "cpu", "frequency_hertz"),
                "CPU frequency in hertz from /proc/cpuinfo.",
                vec!["package".to_string(), "core".to_string(), "cpu".to_string()],
                None,
            )?,
            cpu_flags_info: Desc::new(
                prometheus::core::build_fq_name("namespace", "cpu", "flag_info"),
                "The `flags` field of CPU information from /proc/cpuinfo taken from the first core.",
                vec!["flag".to_string()],
                None,
            )?,
            cpu_bugs_info: Desc::new(
                prometheus::core::build_fq_name("namespace", "cpu", "bug_info"),
                "The `bugs` field of CPU information from /proc/cpuinfo taken from the first core.",
                vec!["bug".to_string()],
                None,
            )?,
            cpu_guest: Desc::new(
                prometheus::core::build_fq_name("namespace", "cpu", "guest_seconds_total"),
                "Seconds the CPUs spent in guests (VMs) for each mode.",
                vec!["cpu".to_string(), "mode".to_string()],
                None,
            )?,
            cpu_core_throttle: Desc::new(
                prometheus::core::build_fq_name("namespace", "cpu", "core_throttles_total"),
                "Number of times this CPU core has been throttled.",
                vec!["package".to_string(), "core".to_string()],
                None,
            )?,
            cpu_package_throttle: Desc::new(
                prometheus::core::build_fq_name("namespace", "cpu", "package_throttles_total"),
                "Number of times this CPU package has been throttled.",
                vec!["package".to_string()],
                None,
            )?,
            cpu_isolated: Desc::new(
                prometheus::core::build_fq_name("namespace", "cpu", "isolated"),
                "Whether each core is isolated, information from /sys/devices/system/cpu/isolated.",
                vec!["cpu".to_string()],
                None,
            )?,
            logger,
            cpu_stats: Mutex::new(HashMap::new()),
            isolated_cpus,
            cpu_flags_include_regexp: None,
            cpu_bugs_include_regexp: None,
        })
    }

    fn compile_include_flags(&mut self, flags_include: &str, bugs_include: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !flags_include.is_empty() {
            self.cpu_flags_include_regexp = Some(Regex::new(flags_include)?);
        }
        if !bugs_include.is_empty() {
            self.cpu_bugs_include_regexp = Some(Regex::new(bugs_include)?);
        }
        Ok(())
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        self.update_info(ch)?;
        self.update_stat(ch)?;
        self.update_isolated(ch);
        self.update_thermal_throttle(ch)?;
        Ok(())
    }

    fn update_info(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let info = self.fs.cpu_info()?;
        for cpu in &info {
            ch(prometheus::core::MetricFamily::new(
                self.cpu_info.clone(),
                prometheus::proto::MetricType::GAUGE,
                1.0,
                vec![cpu.physical_id.clone(), cpu.core_id.clone(), cpu.processor.to_string(), cpu.vendor_id.clone(), cpu.cpu_family.clone(), cpu.model.clone(), cpu.model_name.clone(), cpu.microcode.clone(), cpu.stepping.clone(), cpu.cache_size.clone()],
            ));
        }

        for cpu in &info {
            ch(prometheus::core::MetricFamily::new(
                self.cpu_frequency_hz.clone(),
                prometheus::proto::MetricType::GAUGE,
                cpu.cpu_mhz * 1e6,
                vec![cpu.physical_id.clone(), cpu.core_id.clone(), cpu.processor.to_string()],
            ));
        }

        if let Some(cpu) = info.first() {
            self.update_field_info(&cpu.flags, &self.cpu_flags_include_regexp, &self.cpu_flags_info, ch)?;
            self.update_field_info(&cpu.bugs, &self.cpu_bugs_include_regexp, &self.cpu_bugs_info, ch)?;
        }

        Ok(())
    }

    fn update_field_info(&self, value_list: &[String], filter: &Option<Regex>, desc: &Desc, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(filter) = filter {
            for val in value_list {
                if filter.is_match(val) {
                    ch(prometheus::core::MetricFamily::new(
                        desc.clone(),
                        prometheus::proto::MetricType::GAUGE,
                        1.0,
                        vec![val.clone()],
                    ));
                }
            }
        }
        Ok(())
    }

    fn update_thermal_throttle(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let cpus = glob::glob("/sys/devices/system/cpu/cpu[0-9]*")?;
        let mut package_throttles = HashMap::new();
        let mut package_core_throttles = HashMap::new();

        for cpu in cpus {
            let cpu = cpu?;
            let physical_package_id = read_uint_from_file(cpu.join("topology/physical_package_id"))?;
            let core_id = read_uint_from_file(cpu.join("topology/core_id"))?;

            package_core_throttles.entry(physical_package_id).or_insert_with(HashMap::new).entry(core_id).or_insert_with(|| {
                read_uint_from_file(cpu.join("thermal_throttle/core_throttle_count")).unwrap_or(0)
            });

            package_throttles.entry(physical_package_id).or_insert_with(|| {
                read_uint_from_file(cpu.join("thermal_throttle/package_throttle_count")).unwrap_or(0)
            });
        }

        for (physical_package_id, package_throttle_count) in package_throttles {
            ch(prometheus::core::MetricFamily::new(
                self.cpu_package_throttle.clone(),
                prometheus::proto::MetricType::COUNTER,
                package_throttle_count as f64,
                vec![physical_package_id.to_string()],
            ));
        }

        for (physical_package_id, core_map) in package_core_throttles {
            for (core_id, core_throttle_count) in core_map {
                ch(prometheus::core::MetricFamily::new(
                    self.cpu_core_throttle.clone(),
                    prometheus::proto::MetricType::COUNTER,
                    core_throttle_count as f64,
                    vec![physical_package_id.to_string(), core_id.to_string()],
                ));
            }
        }

        Ok(())
    }

    fn update_isolated(&self, ch: &mut dyn FnMut(MetricFamily)) {
        for &cpu in &self.isolated_cpus {
            ch(prometheus::core::MetricFamily::new(
                self.cpu_isolated.clone(),
                prometheus::proto::MetricType::GAUGE,
                1.0,
                vec![cpu.to_string()],
            ));
        }
    }

    fn update_stat(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let stats = self.fs.stat()?;
        self.update_cpu_stats(stats.cpu);

        let cpu_stats = self.cpu_stats.lock().unwrap();
        for (cpu_id, cpu_stat) in &*cpu_stats {
            let cpu_num = cpu_id.to_string();
            ch(prometheus::core::MetricFamily::new(
                self.cpu.clone(),
                prometheus::proto::MetricType::COUNTER,
                cpu_stat.user,
                vec![cpu_num.clone(), "user".to_string()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.cpu.clone(),
                prometheus::proto::MetricType::COUNTER,
                cpu_stat.nice,
                vec![cpu_num.clone(), "nice".to_string()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.cpu.clone(),
                prometheus::proto::MetricType::COUNTER,
                cpu_stat.system,
                vec![cpu_num.clone(), "system".to_string()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.cpu.clone(),
                prometheus::proto::MetricType::COUNTER,
                cpu_stat.idle,
                vec![cpu_num.clone(), "idle".to_string()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.cpu.clone(),
                prometheus::proto::MetricType::COUNTER,
                cpu_stat.iowait,
                vec![cpu_num.clone(), "iowait".to_string()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.cpu.clone(),
                prometheus::proto::MetricType::COUNTER,
                cpu_stat.irq,
                vec![cpu_num.clone(), "irq".to_string()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.cpu.clone(),
                prometheus::proto::MetricType::COUNTER,
                cpu_stat.softirq,
                vec![cpu_num.clone(), "softirq".to_string()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.cpu.clone(),
                prometheus::proto::MetricType::COUNTER,
                cpu_stat.steal,
                vec![cpu_num.clone(), "steal".to_string()],
            ));

            if *ENABLE_CPU_GUEST {
                ch(prometheus::core::MetricFamily::new(
                    self.cpu_guest.clone(),
                    prometheus::proto::MetricType::COUNTER,
                    cpu_stat.guest,
                    vec![cpu_num.clone(), "user".to_string()],
                ));
                ch(prometheus::core::MetricFamily::new(
                    self.cpu_guest.clone(),
                    prometheus::proto::MetricType::COUNTER,
                    cpu_stat.guest_nice,
                    vec![cpu_num.clone(), "nice".to_string()],
                ));
            }
        }

        Ok(())
    }

    fn update_cpu_stats(&self, new_stats: HashMap<i64, procfs::CPUStat>) {
        let mut cpu_stats = self.cpu_stats.lock().unwrap();

        for (i, n) in new_stats {
            let mut cpu_stat = cpu_stats.entry(i).or_insert(procfs::CPUStat::default());

            if (cpu_stat.idle - n.idle) >= JUMP_BACK_SECONDS {
                self.logger.debug("CPU Idle counter jumped backwards", o!("cpu" => i, "old_value" => cpu_stat.idle, "new_value" => n.idle));
                *cpu_stat = procfs::CPUStat::default();
            }

            if n.idle >= cpu_stat.idle {
                cpu_stat.idle = n.idle;
            } else {
                self.logger.debug("CPU Idle counter jumped backwards", o!("cpu" => i, "old_value" => cpu_stat.idle, "new_value" => n.idle));
            }

            if n.user >= cpu_stat.user {
                cpu_stat.user = n.user;
            } else {
                self.logger.debug("CPU User counter jumped backwards", o!("cpu" => i, "old_value" => cpu_stat.user, "new_value" => n.user));
            }

            if n.nice >= cpu_stat.nice {
                cpu_stat.nice = n.nice;
            } else {
                self.logger.debug("CPU Nice counter jumped backwards", o!("cpu" => i, "old_value" => cpu_stat.nice, "new_value" => n.nice));
            }

            if n.system >= cpu_stat.system {
                cpu_stat.system = n.system;
            } else {
                self.logger.debug("CPU System counter jumped backwards", o!("cpu" => i, "old_value" => cpu_stat.system, "new_value" => n.system));
            }

            if n.iowait >= cpu_stat.iowait {
                cpu_stat.iowait = n.iowait;
            } else {
                self.logger.debug("CPU Iowait counter jumped backwards", o!("cpu" => i, "old_value" => cpu_stat.iowait, "new_value" => n.iowait));
            }

            if n.irq >= cpu_stat.irq {
                cpu_stat.irq = n.irq;
            } else {
                self.logger.debug("CPU IRQ counter jumped backwards", o!("cpu" => i, "old_value" => cpu_stat.irq, "new_value" => n.irq));
            }

            if n.softirq >= cpu_stat.softirq {
                cpu_stat.softirq = n.softirq;
            } else {
                self.logger.debug("CPU SoftIRQ counter jumped backwards", o!("cpu" => i, "old_value" => cpu_stat.softirq, "new_value" => n.softirq));
            }

            if n.steal >= cpu_stat.steal {
                cpu_stat.steal = n.steal;
            } else {
                self.logger.debug("CPU Steal counter jumped backwards", o!("cpu" => i, "old_value" => cpu_stat.steal, "new_value" => n.steal));
            }

            if n.guest >= cpu_stat.guest {
                cpu_stat.guest = n.guest;
            } else {
                self.logger.debug("CPU Guest counter jumped backwards", o!("cpu" => i, "old_value" => cpu_stat.guest, "new_value" => n.guest));
            }

            if n.guest_nice >= cpu_stat.guest_nice {
                cpu_stat.guest_nice = n.guest_nice;
            } else {
                self.logger.debug("CPU GuestNice counter jumped backwards", o!("cpu" => i, "old_value" => cpu_stat.guest_nice, "new_value" => n.guest_nice));
            }
        }

        let online_cpu_ids: Vec<_> = new_stats.keys().cloned().collect();
        cpu_stats.retain(|&key, _| online_cpu_ids.contains(&key));
    }
}

impl Collector for CpuCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&self.cpu);
        descs(&self.cpu_info);
        descs(&self.cpu_frequency_hz);
        descs(&self.cpu_flags_info);
        descs(&self.cpu_bugs_info);
        descs(&self.cpu_guest);
        descs(&self.cpu_core_throttle);
        descs(&self.cpu_package_throttle);
        descs(&self.cpu_isolated);
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

fn read_uint_from_file<P: AsRef<Path>>(path: P) -> Result<u64, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    content.trim().parse().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}