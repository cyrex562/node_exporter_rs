use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::sync::Arc;

const BUDDY_INFO_SUBSYSTEM: &str = "buddyinfo";

struct BuddyInfoCollector {
    fs: procfs::ProcFs,
    desc: Arc<Desc>,
    logger: Logger,
}

impl BuddyInfoCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        let desc = Desc::new(
            prometheus::core::build_fq_name("namespace", BUDDY_INFO_SUBSYSTEM, "blocks"),
            "Count of free blocks according to size.",
            vec!["node".to_string(), "zone".to_string(), "size".to_string()],
            None,
        )?;
        let fs = procfs::ProcFs::new()?;
        Ok(Self { fs, desc: Arc::new(desc), logger })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let buddy_info = self.fs.buddy_info()?;
        self.logger.debug("Set node_buddy", o!("buddyInfo" => format!("{:?}", buddy_info)));
        for entry in buddy_info {
            for (size, value) in entry.sizes.iter() {
                ch(prometheus::core::MetricFamily::new(
                    self.desc.clone(),
                    prometheus::proto::MetricType::GAUGE,
                    value.clone(),
                    vec![entry.node.clone(), entry.zone.clone(), size.to_string()],
                ));
            }
        }
        Ok(())
    }
}

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
struct BuddyInfo {
    node: String,
    zone: String,
    sizes: Vec<f64>,
}

impl FS {
    fn buddy_info(&self) -> Result<Vec<BuddyInfo>, io::Error> {
        let file = File::open(self.proc.path("buddyinfo"))?;
        parse_buddy_info(file)
    }
}

fn parse_buddy_info<R: BufRead>(reader: R) -> Result<Vec<BuddyInfo>, io::Error> {
    let mut buddy_info = Vec::new();
    let mut bucket_count = None;

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 4 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid number of fields"));
        }

        let node = parts[1].trim_end_matches(',');
        let zone = parts[3].trim_end_matches(',');
        let array_size = parts.len() - 4;

        if let Some(count) = bucket_count {
            if count != array_size {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Mismatch in number of buddyinfo buckets"));
            }
        } else {
            bucket_count = Some(array_size);
        }

        let sizes: Result<Vec<f64>, _> = parts[4..].iter().map(|&s| f64::from_str(s)).collect();
        let sizes = sizes.map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid value in buddyinfo"))?;

        buddy_info.push(BuddyInfo {
            node: node.to_string(),
            zone: zone.to_string(),
            sizes,
        });
    }

    Ok(buddy_info)
}