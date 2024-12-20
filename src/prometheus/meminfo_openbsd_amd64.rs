use nix::sys::sysctl::{sysctl, sysctl_raw};
use slog::Logger;
use std::collections::HashMap;
use std::error::Error;
use std::mem;
use std::ptr;

const CTL_VFS: i32 = 10;
const VFS_GENERIC: i32 = 0;
const VFS_BCACHESTAT: i32 = 3;

#[repr(C)]
struct Bcachestats {
    numbufs: i64,
    numbufpages: i64,
    numdirtypages: i64,
    numcleanpages: i64,
    pendingwrites: i64,
    pendingreads: i64,
    numwrites: i64,
    numreads: i64,
    cachehits: i64,
    busymapped: i64,
    dmapages: i64,
    highpages: i64,
    delwribufs: i64,
    kvaslots: i64,
    kvaslots_avail: i64,
    highflips: i64,
    highflops: i64,
    dmaflips: i64,
}

struct MeminfoCollector {
    logger: Logger,
}

impl MeminfoCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn Error>> {
        Ok(MeminfoCollector { logger })
    }

    fn get_mem_info(&self) -> Result<HashMap<String, f64>, Box<dyn Error>> {
        let uvmexpb = sysctl_raw("vm.uvmexp")?;
        let uvmexp: &Uvmexp = unsafe { &*(uvmexpb.as_ptr() as *const Uvmexp) };
        let ps = uvmexp.pagesize as f64;

        let mib = [CTL_VFS, VFS_GENERIC, VFS_BCACHESTAT];
        let bcstatsb = sysctl_raw(&mib)?;
        let bcstats: &Bcachestats = unsafe { &*(bcstatsb.as_ptr() as *const Bcachestats) };

        Ok(HashMap::from([
            ("active_bytes".to_string(), ps * uvmexp.active as f64),
            ("cache_bytes".to_string(), ps * bcstats.numbufpages as f64),
            ("free_bytes".to_string(), ps * uvmexp.free as f64),
            ("inactive_bytes".to_string(), ps * uvmexp.inactive as f64),
            ("size_bytes".to_string(), ps * uvmexp.npages as f64),
            ("swap_size_bytes".to_string(), ps * uvmexp.swpages as f64),
            ("swap_used_bytes".to_string(), ps * uvmexp.swpginuse as f64),
            ("swapped_in_pages_bytes_total".to_string(), ps * uvmexp.pgswapin as f64),
            ("swapped_out_pages_bytes_total".to_string(), ps * uvmexp.pgswapout as f64),
            ("wired_bytes".to_string(), ps * uvmexp.wired as f64),
        ]))
    }
}

#[repr(C)]
struct Uvmexp {
    pagesize: i32,
    active: i32,
    free: i32,
    inactive: i32,
    npages: i32,
    swpages: i32,
    swpginuse: i32,
    pgswapin: i32,
    pgswapout: i32,
    wired: i32,
}