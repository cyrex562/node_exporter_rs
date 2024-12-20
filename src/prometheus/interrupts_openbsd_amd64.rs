use prometheus::{self, core::{Collector, Desc, Metric, Opts, ValueType}};
use slog::Logger;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;
use std::str;
use libc::{c_int, c_uint, c_void, sysctl, CTL_KERN};

const KERN_INTRCNT: c_int = 63;
const KERN_INTRCNT_NUM: c_int = 1;
const KERN_INTRCNT_CNT: c_int = 2;
const KERN_INTRCNT_NAME: c_int = 3;
const KERN_INTRCNT_VECTOR: c_int = 4;

fn nintr() -> c_int {
    let mut mib = [CTL_KERN, KERN_INTRCNT, KERN_INTRCNT_NUM];
    let mut size = std::mem::size_of::<c_int>();
    let mut n: c_int = 0;
    unsafe {
        if sysctl(mib.as_mut_ptr(), mib.len() as c_uint, &mut n as *mut _ as *mut c_void, &mut size, ptr::null_mut(), 0) == -1 {
            return 0;
        }
    }
    n
}

fn intr(idx: c_int) -> Result<Interrupt, String> {
    let mut mib = [CTL_KERN, KERN_INTRCNT, KERN_INTRCNT_NAME, idx];
    let mut size = 128;
    let mut dev = [0u8; 128];
    unsafe {
        if sysctl(mib.as_mut_ptr(), mib.len() as c_uint, dev.as_mut_ptr() as *mut c_void, &mut size, ptr::null_mut(), 0) == -1 {
            return Err("sysctl KERN_INTRCNT_NAME failed".to_string());
        }
    }
    let device = str::from_utf8(&dev).unwrap().trim_end_matches('\0').to_string();

    mib[2] = KERN_INTRCNT_VECTOR;
    let mut vector: c_int = 0;
    size = std::mem::size_of::<c_int>();
    unsafe {
        if sysctl(mib.as_mut_ptr(), mib.len() as c_uint, &mut vector as *mut _ as *mut c_void, &mut size, ptr::null_mut(), 0) == -1 {
            return Err("sysctl KERN_INTRCNT_VECTOR failed".to_string());
        }
    }

    mib[2] = KERN_INTRCNT_CNT;
    let mut count: u64 = 0;
    size = std::mem::size_of::<u64>();
    unsafe {
        if sysctl(mib.as_mut_ptr(), mib.len() as c_uint, &mut count as *mut _ as *mut c_void, &mut size, ptr::null_mut(), 0) == -1 {
            return Err("sysctl KERN_INTRCNT_CNT failed".to_string());
        }
    }

    Ok(Interrupt {
        vector,
        device,
        values: vec![count as f64],
    })
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
    let n = nintr();

    for i in 0..n {
        let itr = intr(i)?;
        interrupts.insert(itr.device.clone(), itr);
    }

    Ok(interrupts)
}