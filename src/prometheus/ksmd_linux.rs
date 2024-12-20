use prometheus::{self, core::{Collector, Desc, Metric, Opts, ValueType}};
use slog::Logger;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const KSMD_FILES: &[&str] = &[
    "full_scans", "merge_across_nodes", "pages_shared", "pages_sharing",
    "pages_to_scan", "pages_unshared", "pages_volatile", "run", "sleep_millisecs"
];

struct KsmdCollector {
    metric_descs: HashMap<String, Desc>,
    logger: Logger,
}

impl KsmdCollector {
    fn new(logger: Logger) -> Result<Self, String> {
        let subsystem = "ksmd";
        let mut descs = HashMap::new();

        for &n in KSMD_FILES {
            descs.insert(n.to_string(), Desc::new(
                format!("node_{}_{}", subsystem, get_canonical_metric_name(n)),
                format!("ksmd '{}' file.", n),
                vec![],
                HashMap::new(),
            ));
        }

        Ok(KsmdCollector { metric_descs: descs, logger })
    }

    fn update(&self, ch: &mut dyn FnMut(Box<dyn Metric>)) -> Result<(), String> {
        for &n in KSMD_FILES {
            let val = read_uint_from_file(&sys_file_path(&format!("kernel/mm/ksm/{}", n)))
                .map_err(|e| format!("failed to read {}: {}", n, e))?;

            let (t, v) = match n {
                "full_scans" => (ValueType::Counter, val as f64),
                "sleep_millisecs" => (ValueType::Gauge, val as f64 / 1000.0),
                _ => (ValueType::Gauge, val as f64),
            };

            ch(Box::new(prometheus::Gauge::new(self.metric_descs[n].clone(), v, vec![])));
        }

        Ok(())
    }
}

fn get_canonical_metric_name(filename: &str) -> &str {
    match filename {
        "full_scans" => "full_scans_total",
        "sleep_millisecs" => "sleep_seconds",
        _ => filename,
    }
}

fn read_uint_from_file(path: &Path) -> Result<u64, std::io::Error> {
    let content = fs::read_to_string(path)?;
    content.trim().parse().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

fn sys_file_path(subpath: &str) -> String {
    format!("/sys/{}", subpath)
}