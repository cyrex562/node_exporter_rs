use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::ffi::CString;
use std::ptr;
use std::sync::Arc;
use std::time::Duration;
use libc::{c_int, c_void};
use std::collections::HashMap;

const CP_USER: usize = 0;
const CP_NICE: usize = 1;
const CP_SYS: usize = 2;
const CP_SPIN: usize = 3;
const CP_INTR: usize = 4;
const CP_IDLE: usize = 5;
const CPUSTATES: usize = 6;

const CP_USER_O63: usize = 0;
const CP_NICE_O63: usize = 1;
const CP_SYS_O63: usize = 2;
const CP_INTR_O63: usize = 3;
const CP_IDLE_O63: usize = 4;
const CPUSTATES_O63: usize = 5;

struct CpuCollector {
    cpu: TypedDesc,
    logger: Logger,
}

impl CpuCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            cpu: TypedDesc::new("node_cpu_seconds_total", "Seconds the CPUs spent in each mode.", vec!["cpu", "mode"]),
            logger,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let clockb = unsafe { sysctl_raw("kern.clockrate")? };
        let clock = unsafe { *(clockb.as_ptr() as *const libc::clockinfo) };
        let hz = clock.stathz as f64;

        let ncpus = unsafe { sysctl_uint32("hw.ncpu")? };

        let mut cp_time = Vec::new();
        for i in 0..ncpus {
            let cpb = unsafe { sysctl_raw(&format!("kern.cp_time2.{}", i))? };
            if cpb.is_empty() {
                continue;
            }
            let mut times = [0u64; CPUSTATES];
            for (n, chunk) in cpb.chunks_exact(8).enumerate() {
                times[n] = u64::from_ne_bytes(chunk.try_into().unwrap());
            }
            if cpb.len() / 8 == CPUSTATES_O63 {
                times[CP_INTR..].copy_from_slice(&times[CP_INTR_O63..]);
                times[CP_SPIN] = 0;
            }
            cp_time.push(times);
        }

        for (cpu, time) in cp_time.iter().enumerate() {
            let lcpu = cpu.to_string();
            ch(self.cpu.must_new_const_metric(time[CP_USER] as f64 / hz, vec![lcpu.clone(), "user".to_string()]));
            ch(self.cpu.must_new_const_metric(time[CP_NICE] as f64 / hz, vec![lcpu.clone(), "nice".to_string()]));
            ch(self.cpu.must_new_const_metric(time[CP_SYS] as f64 / hz, vec![lcpu.clone(), "system".to_string()]));
            ch(self.cpu.must_new_const_metric(time[CP_SPIN] as f64 / hz, vec![lcpu.clone(), "spin".to_string()]));
            ch(self.cpu.must_new_const_metric(time[CP_INTR] as f64 / hz, vec![lcpu.clone(), "interrupt".to_string()]));
            ch(self.cpu.must_new_const_metric(time[CP_IDLE] as f64 / hz, vec![lcpu.clone(), "idle".to_string()]));
        }
        Ok(())
    }
}

impl Collector for CpuCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&self.cpu.desc);
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

unsafe fn sysctl_raw(name: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut size: libc::size_t = 0;
    let cname = CString::new(name).unwrap();
    if libc::sysctlbyname(cname.as_ptr(), ptr::null_mut(), &mut size, ptr::null_mut(), 0) != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let mut buf = vec![0u8; size];
    if libc::sysctlbyname(cname.as_ptr(), buf.as_mut_ptr() as *mut c_void, &mut size, ptr::null_mut(), 0) != 0 {
        return Err(std::io::Error::last_os_error());
    }
    buf.truncate(size);
    Ok(buf)
}

unsafe fn sysctl_uint32(name: &str) -> Result<u32, std::io::Error> {
    let mut value: u32 = 0;
    let mut size = std::mem::size_of::<u32>();
    let cname = CString::new(name).unwrap();
    if libc::sysctlbyname(cname.as_ptr(), &mut value as *mut _ as *mut c_void, &mut size, ptr::null_mut(), 0) != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(value)
}