use prometheus::{self, core::{Collector, Desc, Metric, Opts, ValueType}};
use slog::Logger;
use std::collections::HashMap;
use std::error::Error;

const MEMINFO_SUBSYSTEM: &str = "memory";

struct MeminfoCollector {
    logger: Logger,
}

impl MeminfoCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn Error>> {
        Ok(MeminfoCollector { logger })
    }

    fn update(&self, ch: &mut dyn FnMut(Box<dyn Metric>)) -> Result<(), Box<dyn Error>> {
        let mem_info = self.get_mem_info()?;
        self.logger.debug("Set node_mem", &["memInfo", &format!("{:?}", mem_info)]);
        
        for (k, v) in mem_info {
            let metric_type = if k.ends_with("_total") {
                ValueType::Counter
            } else {
                ValueType::Gauge
            };
            ch(Box::new(prometheus::Gauge::new(
                Desc::new(
                    format!("node_{}_{}", MEMINFO_SUBSYSTEM, k),
                    format!("Memory information field {}.", k),
                    vec![],
                    HashMap::new(),
                ),
                v,
                vec![],
            )));
        }
        Ok(())
    }

    fn get_mem_info(&self) -> Result<HashMap<String, f64>, Box<dyn Error>> {
        // Platform-specific implementation
        Ok(HashMap::new()) // Placeholder
    }
}

use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::str::FromStr;
use std::collections::HashMap;

#[derive(Default)]
pub struct Meminfo {
    mem_total: Option<u64>,
    mem_free: Option<u64>,
    mem_available: Option<u64>,
    buffers: Option<u64>,
    cached: Option<u64>,
    swap_cached: Option<u64>,
    active: Option<u64>,
    inactive: Option<u64>,
    active_anon: Option<u64>,
    inactive_anon: Option<u64>,
    active_file: Option<u64>,
    inactive_file: Option<u64>,
    unevictable: Option<u64>,
    mlocked: Option<u64>,
    swap_total: Option<u64>,
    swap_free: Option<u64>,
    dirty: Option<u64>,
    writeback: Option<u64>,
    anon_pages: Option<u64>,
    mapped: Option<u64>,
    shmem: Option<u64>,
    slab: Option<u64>,
    s_reclaimable: Option<u64>,
    s_unreclaim: Option<u64>,
    kernel_stack: Option<u64>,
    page_tables: Option<u64>,
    nfs_unstable: Option<u64>,
    bounce: Option<u64>,
    writeback_tmp: Option<u64>,
    commit_limit: Option<u64>,
    committed_as: Option<u64>,
    vmalloc_total: Option<u64>,
    vmalloc_used: Option<u64>,
    vmalloc_chunk: Option<u64>,
    percpu: Option<u64>,
    hardware_corrupted: Option<u64>,
    anon_huge_pages: Option<u64>,
    shmem_huge_pages: Option<u64>,
    shmem_pmd_mapped: Option<u64>,
    cma_total: Option<u64>,
    cma_free: Option<u64>,
    huge_pages_total: Option<u64>,
    huge_pages_free: Option<u64>,
    huge_pages_rsvd: Option<u64>,
    huge_pages_surp: Option<u64>,
    hugepagesize: Option<u64>,
    direct_map_4k: Option<u64>,
    direct_map_2m: Option<u64>,
    direct_map_1g: Option<u64>,
}

pub struct ProcFs {
    proc: String,
}

impl ProcFs {
    pub fn new(proc: &str) -> Self {
        ProcFs { proc: proc.to_string() }
    }

    pub fn meminfo(&self) -> Result<Meminfo, io::Error> {
        let path = Path::new(&self.proc).join("meminfo");
        let data = fs::read_to_string(path)?;
        parse_meminfo(&data)
    }
}

fn parse_meminfo(data: &str) -> Result<Meminfo, io::Error> {
    let mut meminfo = Meminfo::default();
    for line in data.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 2 {
            continue;
        }

        let val = u64::from_str(fields[1]).unwrap_or(0);
        let val_bytes = if fields.len() == 3 && fields[2] == "kB" {
            val * 1024
        } else {
            val
        };

        match fields[0] {
            "MemTotal:" => {
                meminfo.mem_total = Some(val);
                meminfo.mem_total_bytes = Some(val_bytes);
            }
            "MemFree:" => {
                meminfo.mem_free = Some(val);
                meminfo.mem_free_bytes = Some(val_bytes);
            }
            "MemAvailable:" => {
                meminfo.mem_available = Some(val);
                meminfo.mem_available_bytes = Some(val_bytes);
            }
            "Buffers:" => {
                meminfo.buffers = Some(val);
                meminfo.buffers_bytes = Some(val_bytes);
            }
            "Cached:" => {
                meminfo.cached = Some(val);
                meminfo.cached_bytes = Some(val_bytes);
            }
            "SwapCached:" => {
                meminfo.swap_cached = Some(val);
                meminfo.swap_cached_bytes = Some(val_bytes);
            }
            "Active:" => {
                meminfo.active = Some(val);
                meminfo.active_bytes = Some(val_bytes);
            }
            "Inactive:" => {
                meminfo.inactive = Some(val);
                meminfo.inactive_bytes = Some(val_bytes);
            }
            "Active(anon):" => {
                meminfo.active_anon = Some(val);
                meminfo.active_anon_bytes = Some(val_bytes);
            }
            "Inactive(anon):" => {
                meminfo.inactive_anon = Some(val);
                meminfo.inactive_anon_bytes = Some(val_bytes);
            }
            "Active(file):" => {
                meminfo.active_file = Some(val);
                meminfo.active_file_bytes = Some(val_bytes);
            }
            "Inactive(file):" => {
                meminfo.inactive_file = Some(val);
                meminfo.inactive_file_bytes = Some(val_bytes);
            }
            "Unevictable:" => {
                meminfo.unevictable = Some(val);
                meminfo.unevictable_bytes = Some(val_bytes);
            }
            "Mlocked:" => {
                meminfo.mlocked = Some(val);
                meminfo.mlocked_bytes = Some(val_bytes);
            }
            "SwapTotal:" => {
                meminfo.swap_total = Some(val);
                meminfo.swap_total_bytes = Some(val_bytes);
            }
            "SwapFree:" => {
                meminfo.swap_free = Some(val);
                meminfo.swap_free_bytes = Some(val_bytes);
            }
            "Dirty:" => {
                meminfo.dirty = Some(val);
                meminfo.dirty_bytes = Some(val_bytes);
            }
            "Writeback:" => {
                meminfo.writeback = Some(val);
                meminfo.writeback_bytes = Some(val_bytes);
            }
            "AnonPages:" => {
                meminfo.anon_pages = Some(val);
                meminfo.anon_pages_bytes = Some(val_bytes);
            }
            "Mapped:" => {
                meminfo.mapped = Some(val);
                meminfo.mapped_bytes = Some(val_bytes);
            }
            "Shmem:" => {
                meminfo.shmem = Some(val);
                meminfo.shmem_bytes = Some(val_bytes);
            }
            "Slab:" => {
                meminfo.slab = Some(val);
                meminfo.slab_bytes = Some(val_bytes);
            }
            "SReclaimable:" => {
                meminfo.s_reclaimable = Some(val);
                meminfo.s_reclaimable_bytes = Some(val_bytes);
            }
            "SUnreclaim:" => {
                meminfo.s_unreclaim = Some(val);
                meminfo.s_unreclaim_bytes = Some(val_bytes);
            }
            "KernelStack:" => {
                meminfo.kernel_stack = Some(val);
                meminfo.kernel_stack_bytes = Some(val_bytes);
            }
            "PageTables:" => {
                meminfo.page_tables = Some(val);
                meminfo.page_tables_bytes = Some(val_bytes);
            }
            "NFS_Unstable:" => {
                meminfo.nfs_unstable = Some(val);
                meminfo.nfs_unstable_bytes = Some(val_bytes);
            }
            "Bounce:" => {
                meminfo.bounce = Some(val);
                meminfo.bounce_bytes = Some(val_bytes);
            }
            "WritebackTmp:" => {
                meminfo.writeback_tmp = Some(val);
                meminfo.writeback_tmp_bytes = Some(val_bytes);
            }
            "CommitLimit:" => {
                meminfo.commit_limit = Some(val);
                meminfo.commit_limit_bytes = Some(val_bytes);
            }
            "Committed_AS:" => {
                meminfo.committed_as = Some(val);
                meminfo.committed_as_bytes = Some(val_bytes);
            }
            "VmallocTotal:" => {
                meminfo.vmalloc_total = Some(val);
                meminfo.vmalloc_total_bytes = Some(val_bytes);
            }
            "VmallocUsed:" => {
                meminfo.vmalloc_used = Some(val);
                meminfo.vmalloc_used_bytes = Some(val_bytes);
            }
            "VmallocChunk:" => {
                meminfo.vmalloc_chunk = Some(val);
                meminfo.vmalloc_chunk_bytes = Some(val_bytes);
            }
            "Percpu:" => {
                meminfo.percpu = Some(val);
                meminfo.percpu_bytes = Some(val_bytes);
            }
            "HardwareCorrupted:" => {
                meminfo.hardware_corrupted = Some(val);
                meminfo.hardware_corrupted_bytes = Some(val_bytes);
            }
            "AnonHugePages:" => {
                meminfo.anon_huge_pages = Some(val);
                meminfo.anon_huge_pages_bytes = Some(val_bytes);
            }
            "ShmemHugePages:" => {
                meminfo.shmem_huge_pages = Some(val);
                meminfo.shmem_huge_pages_bytes = Some(val_bytes);
            }
            "ShmemPmdMapped:" => {
                meminfo.shmem_pmd_mapped = Some(val);
                meminfo.shmem_pmd_mapped_bytes = Some(val_bytes);
            }
            "CmaTotal:" => {
                meminfo.cma_total = Some(val);
                meminfo.cma_total_bytes = Some(val_bytes);
            }
            "CmaFree:" => {
                meminfo.cma_free = Some(val);
                meminfo.cma_free_bytes = Some(val_bytes);
            }
            "HugePages_Total:" => {
                meminfo.huge_pages_total = Some(val);
            }
            "HugePages_Free:" => {
                meminfo.huge_pages_free = Some(val);
            }
            "HugePages_Rsvd:" => {
                meminfo.huge_pages_rsvd = Some(val);
            }
            "HugePages_Surp:" => {
                meminfo.huge_pages_surp = Some(val);
            }
            "Hugepagesize:" => {
                meminfo.hugepagesize = Some(val);
                meminfo.hugepagesize_bytes = Some(val_bytes);
            }
            "DirectMap4k:" => {
                meminfo.direct_map_4k = Some(val);
                meminfo.direct_map_4k_bytes = Some(val_bytes);
            }
            "DirectMap2M:" => {
                meminfo.direct_map_2m = Some(val);
                meminfo.direct_map_2m_bytes = Some(val_bytes);
            }
            "DirectMap1G:" => {
                meminfo.direct_map_1g = Some(val);
                meminfo.direct_map_1g_bytes = Some(val_bytes);
            }
            _ => (),
        }
    }

    Ok(meminfo)
}