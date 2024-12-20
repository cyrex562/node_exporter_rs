use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::ffi::CString;
use std::ptr;
use std::sync::Arc;
use libc::{c_char, c_int, c_void};
use std::mem;

const DISKSTATS_DEFAULT_IGNORED_DEVICES: &str = "";

#[repr(C)]
struct DiskStats {
    ds_name: [c_char; 16],
    ds_busy: i32,
    ds_rxfer: u64,
    ds_wxfer: u64,
    ds_seek: u64,
    ds_rbytes: u64,
    ds_wbytes: u64,
    ds_attachtime: libc::timeval,
    ds_timestamp: libc::timeval,
    ds_time: libc::timeval,
}

struct DiskstatsCollector {
    rxfer: TypedDesc,
    rbytes: TypedDesc,
    wxfer: TypedDesc,
    wbytes: TypedDesc,
    time: TypedDesc,
    device_filter: DeviceFilter,
    logger: Logger,
}

impl DiskstatsCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        let device_filter = new_diskstats_device_filter(&logger)?;

        Ok(Self {
            rxfer: TypedDesc::new("reads_completed_total", "The total number of reads completed successfully.", vec!["device"], prometheus::proto::MetricType::COUNTER),
            rbytes: TypedDesc::new("read_bytes_total", "The total number of bytes read successfully.", vec!["device"], prometheus::proto::MetricType::COUNTER),
            wxfer: TypedDesc::new("writes_completed_total", "The total number of writes completed successfully.", vec!["device"], prometheus::proto::MetricType::COUNTER),
            wbytes: TypedDesc::new("written_bytes_total", "The total number of bytes written successfully.", vec!["device"], prometheus::proto::MetricType::COUNTER),
            time: TypedDesc::new("io_time_seconds_total", "Total seconds spent doing I/Os.", vec!["device"], prometheus::proto::MetricType::COUNTER),
            device_filter,
            logger,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let diskstatsb = unsafe { sysctl_raw("hw.diskstats")? };
        let ndisks = diskstatsb.len() / mem::size_of::<DiskStats>();
        let diskstats: &[DiskStats] = unsafe { std::slice::from_raw_parts(diskstatsb.as_ptr() as *const DiskStats, ndisks) };

        for stat in diskstats {
            let diskname = unsafe { CStr::from_ptr(stat.ds_name.as_ptr()) }.to_str()?;
            if self.device_filter.ignored(diskname) {
                continue;
            }

            ch(self.rxfer.new_metric(stat.ds_rxfer as f64, vec![diskname.to_string()]));
            ch(self.rbytes.new_metric(stat.ds_rbytes as f64, vec![diskname.to_string()]));
            ch(self.wxfer.new_metric(stat.ds_wxfer as f64, vec![diskname.to_string()]));
            ch(self.wbytes.new_metric(stat.ds_wbytes as f64, vec![diskname.to_string()]));
            let time = stat.ds_time.tv_sec as f64 + stat.ds_time.tv_usec as f64 / 1_000_000.0;
            ch(self.time.new_metric(time, vec![diskname.to_string()]));
        }
        Ok(())
    }
}

impl Collector for DiskstatsCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&self.rxfer.desc);
        descs(&self.rbytes.desc);
        descs(&self.wxfer.desc);
        descs(&self.wbytes.desc);
        descs(&self.time.desc);
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        let mut ch = |metric: MetricFamily| {
            metrics(Box::new(metric));
        };
        if let Err(e) = self.update(&mut ch) {
            self.logger.error("failed to collect disk stats", o!("error" => e.to_string()));
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