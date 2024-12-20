use prometheus::{self, core::{Collector, Desc, Metric, Opts, ValueType}};
use slog::Logger;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use regex::Regex;

const MEMINFO_NUMA_SUBSYSTEM: &str = "memory_numa";
lazy_static! {
    static ref MEMINFO_NODE_RE: Regex = Regex::new(r".*devices/system/node/node([0-9]*)").unwrap();
}

struct MeminfoMetric {
    metric_name: String,
    metric_type: ValueType,
    numa_node: String,
    value: f64,
}

struct MeminfoNumaCollector {
    metric_descs: HashMap<String, Desc>,
    logger: Logger,
}

impl MeminfoNumaCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(MeminfoNumaCollector {
            metric_descs: HashMap::new(),
            logger,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(Box<dyn Metric>)) -> Result<(), Box<dyn std::error::Error>> {
        let metrics = get_mem_info_numa()?;
        for v in metrics {
            let desc = self.metric_descs.entry(v.metric_name.clone()).or_insert_with(|| {
                Desc::new(
                    format!("node_{}_{}", MEMINFO_NUMA_SUBSYSTEM, v.metric_name),
                    format!("Memory information field {}.", v.metric_name),
                    vec!["node".to_string()],
                    HashMap::new(),
                )
            });
            ch(Box::new(prometheus::Gauge::new(desc.clone(), v.value, vec![v.numa_node.clone()])));
        }
        Ok(())
    }
}

fn get_mem_info_numa() -> Result<Vec<MeminfoMetric>, Box<dyn std::error::Error>> {
    let mut metrics = Vec::new();
    let nodes = glob::glob("/sys/devices/system/node/node[0-9]*")?;

    for node in nodes {
        let node = node?;
        let meminfo_file = fs::File::open(node.join("meminfo"))?;
        let numa_info = parse_mem_info_numa(meminfo_file)?;
        metrics.extend(numa_info);

        let numastat_file = fs::File::open(node.join("numastat"))?;
        let node_number = MEMINFO_NODE_RE.captures(node.to_str().unwrap())
            .ok_or_else(|| format!("device node string didn't match regexp: {}", node.display()))?[1].to_string();
        let numa_stat = parse_mem_info_numa_stat(numastat_file, &node_number)?;
        metrics.extend(numa_stat);
    }

    Ok(metrics)
}

fn parse_mem_info_numa<R: BufRead>(reader: R) -> Result<Vec<MeminfoMetric>, Box<dyn std::error::Error>> {
    let mut mem_info = Vec::new();
    let re = Regex::new(r"\((.*)\)").unwrap();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let mut value = parts[3].parse::<f64>()?;
        if parts.len() == 5 && parts[4] == "kB" {
            value *= 1024.0;
        }

        let metric = re.replace_all(parts[2].trim_end_matches(':'), "_$1").to_string();
        mem_info.push(MeminfoMetric {
            metric_name: metric,
            metric_type: ValueType::Gauge,
            numa_node: parts[1].to_string(),
            value,
        });
    }

    Ok(mem_info)
}

fn parse_mem_info_numa_stat<R: BufRead>(reader: R, node_number: &str) -> Result<Vec<MeminfoMetric>, Box<dyn std::error::Error>> {
    let mut numa_stat = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(format!("line scan did not return 2 fields: {}", line).into());
        }

        let value = parts[1].parse::<f64>()?;
        numa_stat.push(MeminfoMetric {
            metric_name: format!("{}_total", parts[0]),
            metric_type: ValueType::Counter,
            numa_node: node_number.to_string(),
            value,
        });
    }

    Ok(numa_stat)
}