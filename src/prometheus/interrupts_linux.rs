use prometheus::{self, core::{Collector, Desc, Metric, Opts, ValueType}};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use slog::Logger;

lazy_static! {
    static ref INTERRUPT_LABEL_NAMES: Vec<&'static str> = vec!["cpu", "type", "info", "devices"];
}

struct Interrupt {
    info: String,
    devices: String,
    values: Vec<String>,
}

struct InterruptsCollector {
    desc: TypedDesc,
    logger: Logger,
    name_filter: DeviceFilter,
    include_zeros: bool,
}

impl InterruptsCollector {
    fn update(&self, ch: &mut dyn FnMut(Box<dyn Metric>)) -> Result<(), String> {
        let interrupts = get_interrupts().map_err(|e| format!("couldn't get interrupts: {}", e))?;
        for (name, interrupt) in interrupts {
            for (cpu_no, value) in interrupt.values.iter().enumerate() {
                let filter_name = format!("{};{};{}", name, interrupt.info, interrupt.devices);
                if self.name_filter.ignored(&filter_name) {
                    self.logger.debug("ignoring interrupt name", &["filter_name", &filter_name]);
                    continue;
                }
                let fv: f64 = value.parse().map_err(|e| format!("invalid value {} in interrupts: {}", value, e))?;
                if !self.include_zeros && fv == 0.0 {
                    self.logger.debug("ignoring interrupt with zero value", &["filter_name", &filter_name, "cpu", &cpu_no.to_string()]);
                    continue;
                }
                ch(Box::new(prometheus::Counter::new(self.desc.clone(), fv, vec![cpu_no.to_string(), name.clone(), interrupt.info.clone(), interrupt.devices.clone()])));
            }
        }
        Ok(())
    }
}

fn get_interrupts() -> Result<HashMap<String, Interrupt>, io::Error> {
    let file = File::open("/proc/interrupts")?;
    parse_interrupts(file)
}

fn parse_interrupts<R: BufRead>(reader: R) -> Result<HashMap<String, Interrupt>, io::Error> {
    let mut interrupts = HashMap::new();
    let mut lines = reader.lines();

    if let Some(header) = lines.next() {
        let cpu_num = header?.split_whitespace().count();

        for line in lines {
            let line = line?;
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() > 1 {
                let fields: Vec<&str> = parts[1].split_whitespace().collect();
                if fields.len() < cpu_num + 1 {
                    continue;
                }
                let int_name = parts[0].trim().to_string();
                let mut intr = Interrupt {
                    values: fields[0..cpu_num].iter().map(|s| s.to_string()).collect(),
                    info: String::new(),
                    devices: String::new(),
                };

                if int_name.parse::<i32>().is_ok() {
                    intr.info = fields[cpu_num].to_string();
                    intr.devices = fields[cpu_num + 1..].join(" ");
                } else {
                    intr.info = fields[cpu_num..].join(" ");
                }
                interrupts.insert(int_name, intr);
            }
        }
    }

    Ok(interrupts)
}