use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::ffi::CString;
use std::ptr;
use std::sync::Arc;
use std::time::Duration;
use std::os::raw::c_int;
use libc::{c_void, size_t};

#[repr(C)]
struct ClockInfo {
    hz: i32,
    tick: i32,
    spare: i32,
    stathz: i32,
    profhz: i32,
}

#[repr(C)]
struct CpuTime {
    user: f64,
    nice: f64,
    sys: f64,
    intr: f64,
    idle: f64,
}

fn get_cpu_times() -> Result<Vec<CpuTime>, Box<dyn std::error::Error>> {
    const STATES: usize = 5;

    let clockb = unsafe { sysctl_raw("kern.clockrate")? };
    let clock = unsafe { *(clockb.as_ptr() as *const ClockInfo) };
    let cpb = unsafe { sysctl_raw("kern.cp_times")? };

    let cpufreq = if clock.stathz > 0 {
        clock.stathz as f64
    } else {
        clock.hz as f64
    };

    let mut times = Vec::new();
    let mut cpb_slice = &cpb[..];
    while cpb_slice.len() >= std::mem::size_of::<c_int>() {
        let t = unsafe { *(cpb_slice.as_ptr() as *const c_int) };
        times.push(t as f64 / cpufreq);
        cpb_slice = &cpb_slice[std::mem::size_of::<c_int>()..];
    }

    let mut cpus = vec![CpuTime { user: 0.0, nice: 0.0, sys: 0.0, intr: 0.0, idle: 0.0 }; times.len() / STATES];
    for (i, chunk) in times.chunks_exact(STATES).enumerate() {
        cpus[i].user = chunk[0];
        cpus[i].nice = chunk[1];
        cpus[i].sys = chunk[2];
        cpus[i].intr = chunk[3];
        cpus[i].idle = chunk[4];
    }

    Ok(cpus)
}

struct StatCollector {
    cpu: TypedDesc,
    temp: TypedDesc,
    logger: Logger,
}

impl StatCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            cpu: TypedDesc::new("node_cpu_seconds_total", "Seconds the CPUs spent in each mode.", vec!["cpu", "mode"]),
            temp: TypedDesc::new("node_cpu_temperature_celsius", "CPU temperature", vec!["cpu"]),
            logger,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let cpu_times = get_cpu_times()?;
        for (cpu, t) in cpu_times.iter().enumerate() {
            let lcpu = cpu.to_string();
            ch(self.cpu.must_new_const_metric(t.user, vec![lcpu.clone(), "user".to_string()]));
            ch(self.cpu.must_new_const_metric(t.nice, vec![lcpu.clone(), "nice".to_string()]));
            ch(self.cpu.must_new_const_metric(t.sys, vec![lcpu.clone(), "system".to_string()]));
            ch(self.cpu.must_new_const_metric(t.intr, vec![lcpu.clone(), "interrupt".to_string()]));
            ch(self.cpu.must_new_const_metric(t.idle, vec![lcpu.clone(), "idle".to_string()]));

            match sysctl_uint32(&format!("dev.cpu.{}.temperature", cpu)) {
                Ok(temp) => {
                    let temp_celsius = (temp as i32 - 2732) as f64 / 10.0;
                    ch(self.temp.must_new_const_metric(temp_celsius, vec![lcpu]));
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        self.logger.debug("no temperature information for CPU", o!("cpu" => cpu));
                    } else {
                        ch(self.temp.must_new_const_metric(f64::NAN, vec![lcpu]));
                        self.logger.error("failed to query CPU temperature for CPU", o!("cpu" => cpu, "err" => e.to_string()));
                    }
                }
            }
        }
        Ok(())
    }
}

impl Collector for StatCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&self.cpu.desc);
        descs(&self.temp.desc);
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
    let mut size: size_t = 0;
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

fn sysctl_uint32(name: &str) -> Result<u32, std::io::Error> {
    let mut value: u32 = 0;
    let mut size = std::mem::size_of::<u32>();
    let cname = CString::new(name).unwrap();
    if unsafe { libc::sysctlbyname(cname.as_ptr(), &mut value as *mut _ as *mut c_void, &mut size, ptr::null_mut(), 0) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(value)
}