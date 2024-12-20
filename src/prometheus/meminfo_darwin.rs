use nix::sys::sysctl::{sysctl, sysctl_raw};
use slog::Logger;
use std::collections::HashMap;
use std::error::Error;
use std::mem;
use std::ptr;
use std::slice;

#[repr(C)]
struct VmStatistics64 {
    active_count: u64,
    compressor_page_count: u64,
    inactive_count: u64,
    wire_count: u64,
    free_count: u64,
    pageins: u64,
    pageouts: u64,
    internal_page_count: u64,
    purgeable_count: u64,
}

#[repr(C)]
struct XswUsage {
    xsu_total: u64,
    xsu_avail: u64,
    xsu_used: u64,
    xsu_pagesize: u32,
    xsu_encrypted: bool,
}

struct MeminfoCollector {
    logger: Logger,
}

impl MeminfoCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn Error>> {
        Ok(MeminfoCollector { logger })
    }

    fn get_mem_info(&self) -> Result<HashMap<String, f64>, Box<dyn Error>> {
        let host = unsafe { mach_host_self() };
        let mut info_count = HOST_VM_INFO64_COUNT;
        let mut vmstat = VmStatistics64::default();
        let ret = unsafe {
            host_statistics64(
                host,
                HOST_VM_INFO64,
                &mut vmstat as *mut _ as *mut i32,
                &mut info_count,
            )
        };
        if ret != KERN_SUCCESS {
            return Err(format!("Couldn't get memory statistics, host_statistics returned {}", ret).into());
        }

        let totalb = sysctl("hw.memsize")?;
        let total = u64::from_str_radix(&totalb, 10)?;

        let swapraw = sysctl_raw("vm.swapusage")?;
        let swap: &XswUsage = unsafe { &*(swapraw.as_ptr() as *const XswUsage) };

        let page_size = unsafe { get_page_size(host) } as f64;

        Ok(HashMap::from([
            ("active_bytes".to_string(), page_size * vmstat.active_count as f64),
            ("compressed_bytes".to_string(), page_size * vmstat.compressor_page_count as f64),
            ("inactive_bytes".to_string(), page_size * vmstat.inactive_count as f64),
            ("wired_bytes".to_string(), page_size * vmstat.wire_count as f64),
            ("free_bytes".to_string(), page_size * vmstat.free_count as f64),
            ("swapped_in_bytes_total".to_string(), page_size * vmstat.pageins as f64),
            ("swapped_out_bytes_total".to_string(), page_size * vmstat.pageouts as f64),
            ("internal_bytes".to_string(), page_size * vmstat.internal_page_count as f64),
            ("purgeable_bytes".to_string(), page_size * vmstat.purgeable_count as f64),
            ("total_bytes".to_string(), total as f64),
            ("swap_used_bytes".to_string(), swap.xsu_used as f64),
            ("swap_total_bytes".to_string(), swap.xsu_total as f64),
        ]))
    }
}

extern "C" {
    fn mach_host_self() -> i32;
    fn host_statistics64(host: i32, flavor: i32, info: *mut i32, count: *mut i32) -> i32;
    fn get_page_size(host: i32) -> u32;
}

const HOST_VM_INFO64: i32 = 4;
const HOST_VM_INFO64_COUNT: i32 = mem::size_of::<VmStatistics64>() as i32 / mem::size_of::<i32>() as i32;
const KERN_SUCCESS: i32 = 0;