// Copyright 2020 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

lazy_static::lazy_static! {
    static ref NODE_NUMBER_REGEX: Regex = Regex::new(r".*devices/system/node/node([0-9]*)").unwrap();
}

#[derive(Debug, Default)]
pub struct VMStat {
    nr_free_pages: u64,
    nr_zone_inactive_anon: u64,
    nr_zone_active_anon: u64,
    nr_zone_inactive_file: u64,
    nr_zone_active_file: u64,
    nr_zone_unevictable: u64,
    nr_zone_write_pending: u64,
    nr_mlock: u64,
    nr_page_table_pages: u64,
    nr_kernel_stack: u64,
    nr_bounce: u64,
    nr_zspages: u64,
    nr_free_cma: u64,
    numa_hit: u64,
    numa_miss: u64,
    numa_foreign: u64,
    numa_interleave: u64,
    numa_local: u64,
    numa_other: u64,
    nr_inactive_anon: u64,
    nr_active_anon: u64,
    nr_inactive_file: u64,
    nr_active_file: u64,
    nr_unevictable: u64,
    nr_slab_reclaimable: u64,
    nr_slab_unreclaimable: u64,
    nr_isolated_anon: u64,
    nr_isolated_file: u64,
    workingset_nodes: u64,
    workingset_refault: u64,
    workingset_activate: u64,
    workingset_restore: u64,
    workingset_nodereclaim: u64,
    nr_anon_pages: u64,
    nr_mapped: u64,
    nr_file_pages: u64,
    nr_dirty: u64,
    nr_writeback: u64,
    nr_writeback_temp: u64,
    nr_shmem: u64,
    nr_shmem_hugepages: u64,
    nr_shmem_pmdmapped: u64,
    nr_file_hugepages: u64,
    nr_file_pmdmapped: u64,
    nr_anon_transparent_hugepages: u64,
    nr_vmscan_write: u64,
    nr_vmscan_immediate_reclaim: u64,
    nr_dirtied: u64,
    nr_written: u64,
    nr_kernel_misc_reclaimable: u64,
    nr_foll_pin_acquired: u64,
    nr_foll_pin_released: u64,
}

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn vmstat_numa(&self) -> Result<HashMap<i32, VMStat>, io::Error> {
        let mut m = HashMap::new();
        let pattern = format!("{}/devices/system/node/node[0-9]*", self.sys_path);
        let nodes = glob::glob(&pattern)?;

        for node in nodes {
            let node = node?;
            if let Some(caps) = NODE_NUMBER_REGEX.captures(node.to_str().unwrap()) {
                let node_number: i32 = caps[1].parse().unwrap();
                let file = fs::read_to_string(node.join("vmstat"))?;
                let node_stats = parse_vmstat_numa(&file)?;
                m.insert(node_number, node_stats);
            }
        }
        Ok(m)
    }
}

fn parse_vmstat_numa(content: &str) -> Result<VMStat, io::Error> {
    let mut vmstat = VMStat::default();
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("line scan did not return 2 fields: {}", line)));
        }

        let value: u64 = parts[1].parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        match parts[0] {
            "nr_free_pages" => vmstat.nr_free_pages = value,
            "nr_zone_inactive_anon" => vmstat.nr_zone_inactive_anon = value,
            "nr_zone_active_anon" => vmstat.nr_zone_active_anon = value,
            "nr_zone_inactive_file" => vmstat.nr_zone_inactive_file = value,
            "nr_zone_active_file" => vmstat.nr_zone_active_file = value,
            "nr_zone_unevictable" => vmstat.nr_zone_unevictable = value,
            "nr_zone_write_pending" => vmstat.nr_zone_write_pending = value,
            "nr_mlock" => vmstat.nr_mlock = value,
            "nr_page_table_pages" => vmstat.nr_page_table_pages = value,
            "nr_kernel_stack" => vmstat.nr_kernel_stack = value,
            "nr_bounce" => vmstat.nr_bounce = value,
            "nr_zspages" => vmstat.nr_zspages = value,
            "nr_free_cma" => vmstat.nr_free_cma = value,
            "numa_hit" => vmstat.numa_hit = value,
            "numa_miss" => vmstat.numa_miss = value,
            "numa_foreign" => vmstat.numa_foreign = value,
            "numa_interleave" => vmstat.numa_interleave = value,
            "numa_local" => vmstat.numa_local = value,
            "numa_other" => vmstat.numa_other = value,
            "nr_inactive_anon" => vmstat.nr_inactive_anon = value,
            "nr_active_anon" => vmstat.nr_active_anon = value,
            "nr_inactive_file" => vmstat.nr_inactive_file = value,
            "nr_active_file" => vmstat.nr_active_file = value,
            "nr_unevictable" => vmstat.nr_unevictable = value,
            "nr_slab_reclaimable" => vmstat.nr_slab_reclaimable = value,
            "nr_slab_unreclaimable" => vmstat.nr_slab_unreclaimable = value,
            "nr_isolated_anon" => vmstat.nr_isolated_anon = value,
            "nr_isolated_file" => vmstat.nr_isolated_file = value,
            "workingset_nodes" => vmstat.workingset_nodes = value,
            "workingset_refault" => vmstat.workingset_refault = value,
            "workingset_activate" => vmstat.workingset_activate = value,
            "workingset_restore" => vmstat.workingset_restore = value,
            "workingset_nodereclaim" => vmstat.workingset_nodereclaim = value,
            "nr_anon_pages" => vmstat.nr_anon_pages = value,
            "nr_mapped" => vmstat.nr_mapped = value,
            "nr_file_pages" => vmstat.nr_file_pages = value,
            "nr_dirty" => vmstat.nr_dirty = value,
            "nr_writeback" => vmstat.nr_writeback = value,
            "nr_writeback_temp" => vmstat.nr_writeback_temp = value,
            "nr_shmem" => vmstat.nr_shmem = value,
            "nr_shmem_hugepages" => vmstat.nr_shmem_hugepages = value,
            "nr_shmem_pmdmapped" => vmstat.nr_shmem_pmdmapped = value,
            "nr_file_hugepages" => vmstat.nr_file_hugepages = value,
            "nr_file_pmdmapped" => vmstat.nr_file_pmdmapped = value,
            "nr_anon_transparent_hugepages" => vmstat.nr_anon_transparent_hugepages = value,
            "nr_vmscan_write" => vmstat.nr_vmscan_write = value,
            "nr_vmscan_immediate_reclaim" => vmstat.nr_vmscan_immediate_reclaim = value,
            "nr_dirtied" => vmstat.nr_dirtied = value,
            "nr_written" => vmstat.nr_written = value,
            "nr_kernel_misc_reclaimable" => vmstat.nr_kernel_misc_reclaimable = value,
            "nr_foll_pin_acquired" => vmstat.nr_foll_pin_acquired = value,
            "nr_foll_pin_released" => vmstat.nr_foll_pin_released = value,
            _ => {}
        }
    }
    Ok(vmstat)
}