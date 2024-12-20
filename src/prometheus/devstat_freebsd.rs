// devstat_freebsd.rs

use libc::{c_char, c_int, c_uint, c_ulong, uint64_t};
use std::ffi::CString;
use std::ptr;

#[repr(C)]
struct Stats {
    device: [c_char; 16],
    unit: c_int,
    bytes: Bytes,
    transfers: Transfers,
    duration: Duration,
    busy_time: f64,
    blocks: uint64_t,
}

#[repr(C)]
struct Bytes {
    read: uint64_t,
    write: uint64_t,
    free: uint64_t,
}

#[repr(C)]
struct Transfers {
    other: uint64_t,
    read: uint64_t,
    write: uint64_t,
    free: uint64_t,
}

#[repr(C)]
struct Duration {
    other: f64,
    read: f64,
    write: f64,
    free: f64,
}

extern "C" {
    fn devstat_getdevs(arg1: *mut c_void, arg2: *mut StatInfo) -> c_int;
    fn devstat_compute_statistics(
        dev: *const DevStat,
        last_dev: *const DevStat,
        etime: f64,
        args: ...
    ) -> c_int;
}

#[repr(C)]
struct StatInfo {
    dinfo: *mut DevInfo,
}

#[repr(C)]
struct DevInfo {
    numdevs: c_int,
    devices: *mut DevStat,
}

#[repr(C)]
struct DevStat {
    device_name: [c_char; 16],
    unit_number: c_int,
}

pub fn get_stats(info: &mut DevInfo) -> Result<Vec<Stats>, String> {
    let mut current = StatInfo { dinfo: info };

    let result = unsafe { devstat_getdevs(ptr::null_mut(), &mut current) };
    if result == -1 {
        return Err("devstat_getdevs() failed".to_string());
    }

    let numdevs = unsafe { (*current.dinfo).numdevs };
    let mut stats = Vec::with_capacity(numdevs as usize);

    for i in 0..numdevs {
        let mut stat = Stats {
            device: [0; 16],
            unit: 0,
            bytes: Bytes {
                read: 0,
                write: 0,
                free: 0,
            },
            transfers: Transfers {
                other: 0,
                read: 0,
                write: 0,
                free: 0,
            },
            duration: Duration {
                other: 0.0,
                read: 0.0,
                write: 0.0,
                free: 0.0,
            },
            busy_time: 0.0,
            blocks: 0,
        };

        let device = unsafe { &(*(*current.dinfo).devices.add(i as usize)) };
        unsafe {
            ptr::copy_nonoverlapping(
                device.device_name.as_ptr(),
                stat.device.as_mut_ptr(),
                device.device_name.len(),
            );
        }
        stat.unit = device.unit_number;

        unsafe {
            devstat_compute_statistics(
                device,
                ptr::null(),
                1.0,
                DSM_TOTAL_BYTES_READ, &mut stat.bytes.read,
                DSM_TOTAL_BYTES_WRITE, &mut stat.bytes.write,
                DSM_TOTAL_BYTES_FREE, &mut stat.bytes.free,
                DSM_TOTAL_TRANSFERS_OTHER, &mut stat.transfers.other,
                DSM_TOTAL_TRANSFERS_READ, &mut stat.transfers.read,
                DSM_TOTAL_TRANSFERS_WRITE, &mut stat.transfers.write,
                DSM_TOTAL_TRANSFERS_FREE, &mut stat.transfers.free,
                DSM_TOTAL_DURATION_OTHER, &mut stat.duration.other,
                DSM_TOTAL_DURATION_READ, &mut stat.duration.read,
                DSM_TOTAL_DURATION_WRITE, &mut stat.duration.write,
                DSM_TOTAL_DURATION_FREE, &mut stat.duration.free,
                DSM_TOTAL_BUSY_TIME, &mut stat.busy_time,
                DSM_TOTAL_BLOCKS, &mut stat.blocks,
                DSM_NONE,
            );
        }

        stats.push(stat);
    }

    Ok(stats)
}

use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::ffi::CString;
use std::ptr;
use std::sync::{Arc, Mutex};
use libc::{c_char, c_int, c_void, uint64_t};

#[repr(C)]
struct Stats {
    device: [c_char; 16],
    unit: c_int,
    bytes: Bytes,
    transfers: Transfers,
    duration: Duration,
    busy_time: f64,
    blocks: uint64_t,
}

#[repr(C)]
struct Bytes {
    read: uint64_t,
    write: uint64_t,
    free: uint64_t,
}

#[repr(C)]
struct Transfers {
    other: uint64_t,
    read: uint64_t,
    write: uint64_t,
    free: uint64_t,
}

#[repr(C)]
struct Duration {
    other: f64,
    read: f64,
    write: f64,
    free: f64,
}

extern "C" {
    fn _get_stats(info: *mut DevInfo, stats: *mut *mut Stats) -> c_int;
}

#[repr(C)]
struct DevInfo;

const DEVSTAT_SUBSYSTEM: &str = "devstat";

struct DevstatCollector {
    mu: Mutex<()>,
    devinfo: *mut DevInfo,
    bytes: TypedDesc,
    transfers: TypedDesc,
    duration: TypedDesc,
    busy_time: TypedDesc,
    blocks: TypedDesc,
    logger: Logger,
}

impl DevstatCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            mu: Mutex::new(()),
            devinfo: ptr::null_mut(),
            bytes: TypedDesc::new(
                "devstat_bytes_total",
                "The total number of bytes in transactions.",
                vec!["device", "type"],
                prometheus::proto::MetricType::COUNTER,
            ),
            transfers: TypedDesc::new(
                "devstat_transfers_total",
                "The total number of transactions.",
                vec!["device", "type"],
                prometheus::proto::MetricType::COUNTER,
            ),
            duration: TypedDesc::new(
                "devstat_duration_seconds_total",
                "The total duration of transactions in seconds.",
                vec!["device", "type"],
                prometheus::proto::MetricType::COUNTER,
            ),
            busy_time: TypedDesc::new(
                "devstat_busy_time_seconds_total",
                "Total time the device had one or more transactions outstanding in seconds.",
                vec!["device"],
                prometheus::proto::MetricType::COUNTER,
            ),
            blocks: TypedDesc::new(
                "devstat_blocks_transferred_total",
                "The total number of blocks transferred.",
                vec!["device"],
                prometheus::proto::MetricType::COUNTER,
            ),
            logger,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let _guard = self.mu.lock().unwrap();

        let mut stats: *mut Stats = ptr::null_mut();
        let n = unsafe { _get_stats(self.devinfo, &mut stats) };
        if n == -1 {
            return Err("devstat_getdevs failed".into());
        }

        let base = stats as *const c_void;
        for i in 0..n {
            let offset = i as isize * std::mem::size_of::<Stats>() as isize;
            let stat = unsafe { &*(base.offset(offset) as *const Stats) };

            let device = format!("{}{}", unsafe { CStr::from_ptr(stat.device.as_ptr()) }.to_str()?, stat.unit);
            ch(self.bytes.new_metric(stat.bytes.read as f64, vec![device.clone(), "read".to_string()]));
            ch(self.bytes.new_metric(stat.bytes.write as f64, vec![device.clone(), "write".to_string()]));
            ch(self.transfers.new_metric(stat.transfers.other as f64, vec![device.clone(), "other".to_string()]));
            ch(self.transfers.new_metric(stat.transfers.read as f64, vec![device.clone(), "read".to_string()]));
            ch(self.transfers.new_metric(stat.transfers.write as f64, vec![device.clone(), "write".to_string()]));
            ch(self.duration.new_metric(stat.duration.other, vec![device.clone(), "other".to_string()]));
            ch(self.duration.new_metric(stat.duration.read, vec![device.clone(), "read".to_string()]));
            ch(self.duration.new_metric(stat.duration.write, vec![device.clone(), "write".to_string()]));
            ch(self.busy_time.new_metric(stat.busy_time, vec![device.clone()]));
            ch(self.blocks.new_metric(stat.blocks as f64, vec![device]));
        }
        unsafe { libc::free(stats as *mut c_void) };
        Ok(())
    }
}

impl Collector for DevstatCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&self.bytes.desc);
        descs(&self.transfers.desc);
        descs(&self.duration.desc);
        descs(&self.busy_time.desc);
        descs(&self.blocks.desc);
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        let mut ch = |metric: MetricFamily| {
            metrics(Box::new(metric));
        };
        if let Err(e) = self.update(&mut ch) {
            self.logger.error("failed to collect device stats", o!("error" => e.to_string()));
        }
    }
}