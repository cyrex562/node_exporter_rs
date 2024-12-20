use prometheus::{self, core::{Collector, Desc, Metric, Opts, ValueType}};
use slog::Logger;
use std::error::Error;
use std::collections::HashMap;
use nix::sys::sysctl::sysctl;

const MEMORY_SUBSYSTEM: &str = "memory";

struct MemoryCollector {
    page_size: u64,
    sysctls: Vec<BsdSysctl>,
    kvm: Kvm,
    logger: Logger,
}

impl MemoryCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn Error>> {
        let page_size: u64 = sysctl("vm.stats.vm.v_page_size")?;
        let mib_swap_total = match sysctl::<u64>("vm.swap_total") {
            Ok(_) => "vm.swap_total",
            Err(_) => "vm.swap_size",
        };

        let from_page = |v: f64| v * page_size as f64;

        Ok(MemoryCollector {
            logger,
            page_size,
            sysctls: vec![
                BsdSysctl::new("active_bytes", "Recently used by userland", "vm.stats.vm.v_active_count", from_page),
                BsdSysctl::new("inactive_bytes", "Not recently used by userland", "vm.stats.vm.v_inactive_count", from_page),
                BsdSysctl::new("wired_bytes", "Locked in memory by kernel, mlock, etc", "vm.stats.vm.v_wire_count", from_page),
                BsdSysctl::new("user_wired_bytes", "Locked in memory by user, mlock, etc", "vm.stats.vm.v_user_wire_count", from_page).with_data_type(BsdSysctlType::CLong),
                BsdSysctl::new("cache_bytes", "Almost free, backed by swap or files, available for re-allocation", "vm.stats.vm.v_cache_count", from_page),
                BsdSysctl::new("buffer_bytes", "Disk IO Cache entries for non ZFS filesystems, only usable by kernel", "vfs.bufspace", from_page).with_data_type(BsdSysctlType::CLong),
                BsdSysctl::new("free_bytes", "Unallocated, available for allocation", "vm.stats.vm.v_free_count", from_page),
                BsdSysctl::new("laundry_bytes", "Dirty not recently used by userland", "vm.stats.vm.v_laundry_count", from_page),
                BsdSysctl::new("size_bytes", "Total physical memory size", "vm.stats.vm.v_page_count", from_page),
                BsdSysctl::new("swap_size_bytes", "Total swap memory size", mib_swap_total, from_page).with_data_type(BsdSysctlType::Uint64),
                BsdSysctl::new("swap_in_bytes_total", "Bytes paged in from swap devices", "vm.stats.vm.v_swappgsin", from_page).with_value_type(ValueType::Counter),
                BsdSysctl::new("swap_out_bytes_total", "Bytes paged out to swap devices", "vm.stats.vm.v_swappgsout", from_page).with_value_type(ValueType::Counter),
            ],
            kvm: Kvm::new(),
        })
    }

    fn update(&self, ch: &mut dyn FnMut(Box<dyn Metric>)) -> Result<(), Box<dyn Error>> {
        for m in &self.sysctls {
            let v = m.value()?;
            let value_type = m.value_type.unwrap_or(ValueType::Gauge);

            ch(Box::new(prometheus::Gauge::new(
                Desc::new(
                    format!("node_{}_{}", MEMORY_SUBSYSTEM, m.name),
                    m.description.clone(),
                    vec![],
                    HashMap::new(),
                ),
                v,
                vec![],
            )));
        }

        let swap_used = self.kvm.swap_used_pages()?;
        ch(Box::new(prometheus::Gauge::new(
            Desc::new(
                format!("node_{}_swap_used_bytes", MEMORY_SUBSYSTEM),
                "Currently allocated swap".to_string(),
                vec![],
                HashMap::new(),
            ),
            swap_used as f64 * self.page_size as f64,
            vec![],
        )));

        Ok(())
    }
}

struct BsdSysctl {
    name: String,
    description: String,
    mib: String,
    conversion: fn(f64) -> f64,
    value_type: Option<ValueType>,
    data_type: Option<BsdSysctlType>,
}

impl BsdSysctl {
    fn new(name: &str, description: &str, mib: &str, conversion: fn(f64) -> f64) -> Self {
        BsdSysctl {
            name: name.to_string(),
            description: description.to_string(),
            mib: mib.to_string(),
            conversion,
            value_type: None,
            data_type: None,
        }
    }

    fn with_value_type(mut self, value_type: ValueType) -> Self {
        self.value_type = Some(value_type);
        self
    }

    fn with_data_type(mut self, data_type: BsdSysctlType) -> Self {
        self.data_type = Some(data_type);
        self
    }

    fn value(&self) -> Result<f64, Box<dyn Error>> {
        let raw_value: f64 = sysctl(&self.mib)?;
        Ok((self.conversion)(raw_value))
    }
}

enum BsdSysctlType {
    CLong,
    Uint64,
}

struct Kvm;

impl Kvm {
    fn new() -> Self {
        Kvm
    }

    fn swap_used_pages(&self) -> Result<u64, Box<dyn Error>> {
        // Implementation for retrieving swap used pages
        Ok(0) // Placeholder
    }
}