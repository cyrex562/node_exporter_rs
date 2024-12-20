use prometheus::core::{Collector, Desc, Opts, ValueType};
use slog::Logger;
use std::collections::HashMap;
use std::error::Error;

struct MeminfoCollector {
    fs: procfs::ProcFs,
    logger: Logger,
}

impl MeminfoCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn Error>> {
        let fs = procfs::ProcFs::new()?;
        Ok(MeminfoCollector { fs, logger })
    }

    fn get_mem_info(&self) -> Result<HashMap<String, f64>, Box<dyn Error>> {
        let meminfo = self.fs.meminfo()?;
        let mut metrics = HashMap::new();

        if let Some(value) = meminfo.active_bytes {
            metrics.insert("Active_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.active_anon_bytes {
            metrics.insert("Active_anon_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.active_file_bytes {
            metrics.insert("Active_file_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.anon_huge_pages_bytes {
            metrics.insert("AnonHugePages_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.anon_pages_bytes {
            metrics.insert("AnonPages_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.bounce_bytes {
            metrics.insert("Bounce_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.buffers_bytes {
            metrics.insert("Buffers_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.cached_bytes {
            metrics.insert("Cached_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.cma_free_bytes {
            metrics.insert("CmaFree_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.cma_total_bytes {
            metrics.insert("CmaTotal_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.commit_limit_bytes {
            metrics.insert("CommitLimit_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.committed_as_bytes {
            metrics.insert("Committed_AS_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.direct_map_1g_bytes {
            metrics.insert("DirectMap1G_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.direct_map_2m_bytes {
            metrics.insert("DirectMap2M_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.direct_map_4k_bytes {
            metrics.insert("DirectMap4k_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.dirty_bytes {
            metrics.insert("Dirty_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.hardware_corrupted_bytes {
            metrics.insert("HardwareCorrupted_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.hugepagesize_bytes {
            metrics.insert("Hugepagesize_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.inactive_bytes {
            metrics.insert("Inactive_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.inactive_anon_bytes {
            metrics.insert("Inactive_anon_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.inactive_file_bytes {
            metrics.insert("Inactive_file_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.kernel_stack_bytes {
            metrics.insert("KernelStack_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.mapped_bytes {
            metrics.insert("Mapped_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.mem_available_bytes {
            metrics.insert("MemAvailable_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.mem_free_bytes {
            metrics.insert("MemFree_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.mem_total_bytes {
            metrics.insert("MemTotal_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.mlocked_bytes {
            metrics.insert("Mlocked_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.nfs_unstable_bytes {
            metrics.insert("NFS_Unstable_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.page_tables_bytes {
            metrics.insert("PageTables_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.percpu_bytes {
            metrics.insert("Percpu_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.s_reclaimable_bytes {
            metrics.insert("SReclaimable_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.s_unreclaim_bytes {
            metrics.insert("SUnreclaim_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.shmem_bytes {
            metrics.insert("Shmem_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.shmem_huge_pages_bytes {
            metrics.insert("ShmemHugePages_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.shmem_pmd_mapped_bytes {
            metrics.insert("ShmemPmdMapped_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.slab_bytes {
            metrics.insert("Slab_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.swap_cached_bytes {
            metrics.insert("SwapCached_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.swap_free_bytes {
            metrics.insert("SwapFree_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.swap_total_bytes {
            metrics.insert("SwapTotal_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.unevictable_bytes {
            metrics.insert("Unevictable_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.vmalloc_chunk_bytes {
            metrics.insert("VmallocChunk_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.vmalloc_total_bytes {
            metrics.insert("VmallocTotal_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.vmalloc_used_bytes {
            metrics.insert("VmallocUsed_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.writeback_bytes {
            metrics.insert("Writeback_bytes".to_string(), value as f64);
        }
        if let Some(value) = meminfo.writeback_tmp_bytes {
            metrics.insert("WritebackTmp_bytes".to_string(), value as f64);
        }

        if let Some(value) = meminfo.huge_pages_free {
            metrics.insert("HugePages_Free".to_string(), value as f64);
        }
        if let Some(value) = meminfo.huge_pages_rsvd {
            metrics.insert("HugePages_Rsvd".to_string(), value as f64);
        }
        if let Some(value) = meminfo.huge_pages_surp {
            metrics.insert("HugePages_Surp".to_string(), value as f64);
        }
        if let Some(value) = meminfo.huge_pages_total {
            metrics.insert("HugePages_Total".to_string(), value as f64);
        }

        Ok(metrics)
    }
}

mod procfs {
    use std::fs;
    use std::path::Path;
    use std::collections::HashMap;
    use std::io::Error;

    pub struct ProcFs {
        proc: String,
    }

    impl ProcFs {
        pub fn new() -> Result<Self, Error> {
            Ok(ProcFs { proc: "/proc".to_string() })
        }

        pub fn meminfo(&self) -> Result<Meminfo, Error> {
            let path = Path::new(&self.proc).join("meminfo");
            let data = fs::read_to_string(path)?;
            parse_meminfo(&data)
        }
    }

    #[derive(Debug)]
    pub struct Meminfo {
        pub active_bytes: Option<u64>,
        pub active_anon_bytes: Option<u64>,
        pub active_file_bytes: Option<u64>,
        pub anon_huge_pages_bytes: Option<u64>,
        pub anon_pages_bytes: Option<u64>,
        pub bounce_bytes: Option<u64>,
        pub buffers_bytes: Option<u64>,
        pub cached_bytes: Option<u64>,
        pub cma_free_bytes: Option<u64>,
        pub cma_total_bytes: Option<u64>,
        pub commit_limit_bytes: Option<u64>,
        pub committed_as_bytes: Option<u64>,
        pub direct_map_1g_bytes: Option<u64>,
        pub direct_map_2m_bytes: Option<u64>,
        pub direct_map_4k_bytes: Option<u64>,
        pub dirty_bytes: Option<u64>,
        pub hardware_corrupted_bytes: Option<u64>,
        pub hugepagesize_bytes: Option<u64>,
        pub inactive_bytes: Option<u64>,
        pub inactive_anon_bytes: Option<u64>,
        pub inactive_file_bytes: Option<u64>,
        pub kernel_stack_bytes: Option<u64>,
        pub mapped_bytes: Option<u64>,
        pub mem_available_bytes: Option<u64>,
        pub mem_free_bytes: Option<u64>,
        pub mem_total_bytes: Option<u64>,
        pub mlocked_bytes: Option<u64>,
        pub nfs_unstable_bytes: Option<u64>,
        pub page_tables_bytes: Option<u64>,
        pub percpu_bytes: Option<u64>,
        pub s_reclaimable_bytes: Option<u64>,
        pub s_unreclaim_bytes: Option<u64>,
        pub shmem_bytes: Option<u64>,
        pub shmem_huge_pages_bytes: Option<u64>,
        pub shmem_pmd_mapped_bytes: Option<u64>,
        pub slab_bytes: Option<u64>,
        pub swap_cached_bytes: Option<u64>,
        pub swap_free_bytes: Option<u64>,
        pub swap_total_bytes: Option<u64>,
        pub unevictable_bytes: Option<u64>,
        pub vmalloc_chunk_bytes: Option<u64>,
        pub vmalloc_total_bytes: Option<u64>,
        pub vmalloc_used_bytes: Option<u64>,
        pub writeback_bytes: Option<u64>,
        pub writeback_tmp_bytes: Option<u64>,
        pub huge_pages_free: Option<u64>,
        pub huge_pages_rsvd: Option<u64>,
        pub huge_pages_surp: Option<u64>,
        pub huge_pages_total: Option<u64>,
    }

    fn parse_meminfo(data: &str) -> Result<Meminfo, Error> {
        let mut meminfo = Meminfo {
            active_bytes: None,
            active_anon_bytes: None,
            active_file_bytes: None,
            anon_huge_pages_bytes: None,
            anon_pages_bytes: None,
            bounce_bytes: None,
            buffers_bytes: None,
            cached_bytes: None,
            cma_free_bytes: None,
            cma_total_bytes: None,
            commit_limit_bytes: None,
            committed_as_bytes: None,
            direct_map_1g_bytes: None,
            direct_map_2m_bytes: None,
            direct_map_4k_bytes: None,
            dirty_bytes: None,
            hardware_corrupted_bytes: None,
            hugepagesize_bytes: None,
            inactive_bytes: None,
            inactive_anon_bytes: None,
            inactive_file_bytes: None,
            kernel_stack_bytes: None,
            mapped_bytes: None,
            mem_available_bytes: None,
            mem_free_bytes: None,
            mem_total_bytes: None,
            mlocked_bytes: None,
            nfs_unstable_bytes: None,
            page_tables_bytes: None,
            percpu_bytes: None,
            s_reclaimable_bytes: None,
            s_unreclaim_bytes: None,
            shmem_bytes: None,
            shmem_huge_pages_bytes: None,
            shmem_pmd_mapped_bytes: None,
            slab_bytes: None,
            swap_cached_bytes: None,
            swap_free_bytes: None,
            swap_total_bytes: None,
            unevictable_bytes: None,
            vmalloc_chunk_bytes: None,
            vmalloc_total_bytes: None,
            vmalloc_used_bytes: None,
            writeback_bytes: None,
            writeback_tmp_bytes: None,
            huge_pages_free: None,
            huge_pages_rsvd: None,
            huge_pages_surp: None,
            huge_pages_total: None,
        };

        for line in data.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            let key = parts[0].trim_end_matches(':');
            let value = parts[1].parse::<u64>().unwrap_or(0);

            match key {
                "Active" => meminfo.active_bytes = Some(value),
                "Active(anon)" => meminfo.active_anon_bytes = Some(value),
                "Active(file)" => meminfo.active_file_bytes = Some(value),
                "AnonHugePages" => meminfo.anon_huge_pages_bytes = Some(value),
                "AnonPages" => meminfo.anon_pages_bytes = Some(value),
                "Bounce" => meminfo.bounce_bytes = Some(value),
                "Buffers" => meminfo.buffers_bytes = Some(value),
                "Cached" => meminfo.cached_bytes = Some(value),
                "CmaFree" => meminfo.cma_free_bytes = Some(value),
                "CmaTotal" => meminfo.cma_total_bytes = Some(value),
                "CommitLimit" => meminfo.commit_limit_bytes = Some(value),
                "Committed_AS" => meminfo.committed_as_bytes = Some(value),
                "DirectMap1G" => meminfo.direct_map_1g_bytes = Some(value),
                "DirectMap2M" => meminfo.direct_map_2m_bytes = Some(value),
                "DirectMap4k" => meminfo.direct_map_4k_bytes = Some(value),
                "Dirty" => meminfo.dirty_bytes = Some(value),
                "HardwareCorrupted" => meminfo.hardware_corrupted_bytes = Some(value),
                "Hugepagesize" => meminfo.hugepagesize_bytes = Some(value),
                "Inactive" => meminfo.inactive_bytes = Some(value),
                "Inactive(anon)" => meminfo.inactive_anon_bytes = Some(value),
                "Inactive(file)" => meminfo.inactive_file_bytes = Some(value),
                "KernelStack" => meminfo.kernel_stack_bytes = Some(value),
                "Mapped" => meminfo.mapped_bytes = Some(value),
                "MemAvailable" => meminfo.mem_available_bytes = Some(value),
                "MemFree" => meminfo.mem_free_bytes = Some(value),
                "MemTotal" => meminfo.mem_total_bytes = Some(value),
                "Mlocked" => meminfo.mlocked_bytes = Some(value),
                "NFS_Unstable" => meminfo.nfs_unstable_bytes = Some(value),
                "PageTables" => meminfo.page_tables_bytes = Some(value),
                "Percpu" => meminfo.percpu_bytes = Some(value),
                "SReclaimable" => meminfo.s_reclaimable_bytes = Some(value),
                "SUnreclaim" => meminfo.s_unreclaim_bytes = Some(value),
                "Shmem" => meminfo.shmem_bytes = Some(value),
                "ShmemHugePages" => meminfo.shmem_huge_pages_bytes = Some(value),
                "ShmemPmdMapped" => meminfo.shmem_pmd_mapped_bytes = Some(value),
                "Slab" => meminfo.slab_bytes = Some(value),
                "SwapCached" => meminfo.swap_cached_bytes = Some(value),
                "SwapFree" => meminfo.swap_free_bytes = Some(value),
                "SwapTotal" => meminfo.swap_total_bytes = Some(value),
                "Unevictable" => meminfo.unevictable_bytes = Some(value),
                "VmallocChunk" => meminfo.vmalloc_chunk_bytes = Some(value),
                "VmallocTotal" => meminfo.vmalloc_total_bytes = Some(value),
                "VmallocUsed" => meminfo.vmalloc_used_bytes = Some(value),
                "Writeback" => meminfo.writeback_bytes = Some(value),
                "WritebackTmp" => meminfo.writeback_tmp_bytes = Some(value),
                "HugePages_Free" => meminfo.huge_pages_free = Some(value),
                "HugePages_Rsvd" => meminfo.huge_pages_rsvd = Some(value),
                "HugePages_Surp" => meminfo.huge_pages_surp = Some(value),
                "HugePages_Total" => meminfo.huge_pages_total = Some(value),
                _ => (),
            }
        }

        Ok(meminfo)
    }
}