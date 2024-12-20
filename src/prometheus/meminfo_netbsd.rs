use nix::sys::sysctl::sysctl;
use slog::Logger;
use std::collections::HashMap;
use std::error::Error;

struct MeminfoCollector {
    logger: Logger,
}

impl MeminfoCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn Error>> {
        Ok(MeminfoCollector { logger })
    }

    fn get_mem_info(&self) -> Result<HashMap<String, f64>, Box<dyn Error>> {
        let uvmexp: Uvmexp = sysctl("vm.uvmexp2")?;
        let ps = uvmexp.pagesize as f64;

        Ok(HashMap::from([
            ("active_bytes".to_string(), ps * uvmexp.active as f64),
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