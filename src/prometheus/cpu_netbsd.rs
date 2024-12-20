use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;
use std::sync::Arc;
use std::time::Duration;
use libc::{c_int, c_void};
use regex::Regex;

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

    let cpufreq = if clock.stathz > 0 {
        clock.stathz as f64
    } else {
        clock.hz as f64
    };

    let ncpusb = unsafe { sysctl_raw("hw.ncpu")? };
    let ncpus = unsafe { *(ncpusb.as_ptr() as *const i32) };

    if ncpus < 1 {
        return Err("Invalid cpu number".into());
    }

    let mut times = Vec::new();
    for ncpu in 0..ncpus {
        let cpb = unsafe { sysctl_raw(&format!("kern.cp_time.{}", ncpu))? };
        let mut cpb_slice = &cpb[..];
        while cpb_slice.len() >= std::mem::size_of::<c_int>() {
            let t = unsafe { *(cpb_slice.as_ptr() as *const c_int) };
            times.push(t as f64 / cpufreq);
            cpb_slice = &cpb_slice[std::mem::size_of::<c_int>()..];
        }
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

fn get_cpu_temperatures() -> Result<HashMap<i32, f64>, Box<dyn std::error::Error>> {
    let mut res = HashMap::new();
    let props = read_sysmon_properties()?;
    let keys = sort_filter_sysmon_properties(&props, "coretemp");

    for key in keys {
        convert_temperatures(&props[&key], &mut res)?;
    }

    Ok(res)
}

fn read_sysmon_properties() -> Result<HashMap<String, Vec<SysmonValues>>, Box<dyn std::error::Error>> {
    let fd = unsafe { libc::open("/dev/sysmon".as_ptr() as *const i8, libc::O_RDONLY) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let mut retptr = PlistRef { pref_plist: ptr::null_mut(), pref_len: 0 };
    unsafe {
        ioctl(fd, 0, 'E' as u8, std::mem::size_of::<PlistRef>(), &mut retptr as *mut _ as *mut c_void)?;
        libc::close(fd);
    }
    let bytes = read_bytes(retptr.pref_plist, retptr.pref_len);
    let props: HashMap<String, Vec<SysmonValues>> = plist::from_bytes(&bytes)?;
    Ok(props)
}

fn sort_filter_sysmon_properties(props: &HashMap<String, Vec<SysmonValues>>, prefix: &str) -> Vec<String> {
    let mut keys: Vec<String> = props.keys().filter(|key| key.starts_with(prefix)).cloned().collect();
    keys.sort();
    keys
}

fn convert_temperatures(prop: &[SysmonValues], res: &mut HashMap<i32, f64>) -> Result<(), Box<dyn std::error::Error>> {
    let re = Regex::new(r"^cpu([0-9]+) temperature$")?;
    for val in prop {
        if val.state == "invalid" || val.state == "unknown" || val.state.is_empty() {
            continue;
        }
        if let Some(caps) = re.captures(&val.description) {
            let core: i32 = caps[1].parse()?;
            let temperature = (val.cur_value as f64 / 1_000_000.0) - 273.15;
            res.insert(core, temperature);
        }
    }
    Ok(())
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
        let cpu_temperatures = get_cpu_temperatures()?;

        for (cpu, t) in cpu_times.iter().enumerate() {
            let lcpu = cpu.to_string();
            ch(self.cpu.must_new_const_metric(t.user, vec![lcpu.clone(), "user".to_string()]));
            ch(self.cpu.must_new_const_metric(t.nice, vec![lcpu.clone(), "nice".to_string()]));
            ch(self.cpu.must_new_const_metric(t.sys, vec![lcpu.clone(), "system".to_string()]));
            ch(self.cpu.must_new_const_metric(t.intr, vec![lcpu.clone(), "interrupt".to_string()]));
            ch(self.cpu.must_new_const_metric(t.idle, vec![lcpu.clone(), "idle".to_string()]));

            if let Some(temp) = cpu_temperatures.get(&(cpu as i32)) {
                ch(self.temp.must_new_const_metric(*temp, vec![lcpu]));
            } else {
                self.logger.debug("no temperature information for CPU", o!("cpu" => cpu));
                ch(self.temp.must_new_const_metric(f64::NAN, vec![lcpu]));
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

unsafe fn ioctl(fd: c_int, nr: i64, typ: u8, size: usize, retptr: *mut c_void) -> Result<(), std::io::Error> {
    let ret = libc::ioctl(fd, (0x40000000 | 0x80000000 | ((size & ((1 << 13) - 1)) << 16) | ((typ as i64) << 8) | nr) as libc::c_ulong, retptr);
    if ret != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

fn read_bytes(ptr: *mut c_void, length: u64) -> Vec<u8> {
    unsafe { std::slice::from_raw_parts(ptr as *const u8, length as usize).to_vec() }
}

#[derive(Debug, Deserialize)]
struct SysmonValues {
    cur_value: i32,
    description: String,
    state: String,
    #[serde(rename = "type")]
    typ: String,
}

#[derive(Debug)]
struct PlistRef {
    pref_plist: *mut c_void,
    pref_len: u64,
}