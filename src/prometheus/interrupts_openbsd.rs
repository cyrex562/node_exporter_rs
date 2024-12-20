use prometheus::{self, core::{Collector, Desc, Metric, Opts, ValueType}};
use slog::Logger;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::ptr;
use std::str;
use libc::{sysctl, CTL_KERN};

#[repr(C)]
struct Intr {
    vector: c_int,
    device: [c_char; 128],
    count: u64,
}

extern "C" {
    fn sysctl_nintr() -> c_int;
    fn sysctl_intr(intr: *mut Intr, idx: c_int) -> c_int;
}

struct Interrupt {
    vector: c_int,
    device: String,
    values: Vec<f64>,
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
        for (dev, interrupt) in interrupts {
            for (cpu_no, &value) in interrupt.values.iter().enumerate() {
                let interrupt_type = interrupt.vector.to_string();
                let filter_name = format!("{};{}", interrupt_type, dev);
                if self.name_filter.ignored(&filter_name) {
                    self.logger.debug("ignoring interrupt name", &["filter_name", &filter_name]);
                    continue;
                }
                if !self.include_zeros && value == 0.0 {
                    self.logger.debug("ignoring interrupt with zero value", &["filter_name", &filter_name, "cpu", &cpu_no.to_string()]);
                    continue;
                }
                ch(Box::new(prometheus::Counter::new(self.desc.clone(), value, vec![cpu_no.to_string(), interrupt_type, dev.clone()])));
            }
        }
        Ok(())
    }
}

fn get_interrupts() -> Result<HashMap<String, Interrupt>, String> {
    let mut interrupts = HashMap::new();
    let nintr = unsafe { sysctl_nintr() };

    for i in 0..nintr {
        let mut cintr = Intr {
            vector: 0,
            device: [0; 128],
            count: 0,
        };
        let res = unsafe { sysctl_intr(&mut cintr, i) };
        if res < 0 {
            return Err("sysctl_intr failed".to_string());
        }

        let dev = unsafe { CStr::from_ptr(cintr.device.as_ptr()) }.to_str().unwrap().to_string();

        interrupts.insert(dev.clone(), Interrupt {
            vector: cintr.vector,
            device: dev,
            values: vec![cintr.count as f64],
        });
    }

    Ok(interrupts)
}