use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::ffi::CString;
use std::ptr;
use std::sync::Arc;
use libc::{c_char, c_int, uint64_t};

#[repr(C)]
struct Stats {
    device: [c_char; 16],
    unit: c_int,
    bytes: uint64_t,
    transfers: uint64_t,
    blocks: uint64_t,
}

extern "C" {
    fn _get_ndevs() -> c_int;
    fn _get_stats(i: c_int) -> Stats;
}

const DEVSTAT_SUBSYSTEM: &str = "devstat";

struct DevstatCollector {
    bytes_desc: Desc,
    transfers_desc: Desc,
    blocks_desc: Desc,
    logger: Logger,
}

impl DevstatCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            bytes_desc: Desc::new(
                prometheus::core::build_fq_name("namespace", DEVSTAT_SUBSYSTEM, "bytes_total"),
                "The total number of bytes transferred for reads and writes on the device.",
                vec!["device".to_string()],
                None,
            )?,
            transfers_desc: Desc::new(
                prometheus::core::build_fq_name("namespace", DEVSTAT_SUBSYSTEM, "transfers_total"),
                "The total number of transactions completed.",
                vec!["device".to_string()],
                None,
            )?,
            blocks_desc: Desc::new(
                prometheus::core::build_fq_name("namespace", DEVSTAT_SUBSYSTEM, "blocks_total"),
                "The total number of bytes given in terms of the devices blocksize.",
                vec!["device".to_string()],
                None,
            )?,
            logger,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let count = unsafe { _get_ndevs() };
        if count == -1 {
            return Err("getdevs() failed".into());
        }
        if count == -2 {
            return Err("calloc() failed".into());
        }

        for i in 0..count {
            let stats = unsafe { _get_stats(i) };
            let device = format!("{}{}", unsafe { CStr::from_ptr(stats.device.as_ptr()) }.to_str()?, stats.unit);

            ch(prometheus::core::MetricFamily::new(
                self.bytes_desc.clone(),
                prometheus::proto::MetricType::COUNTER,
                stats.bytes as f64,
                vec![device.clone()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.transfers_desc.clone(),
                prometheus::proto::MetricType::COUNTER,
                stats.transfers as f64,
                vec![device.clone()],
            ));
            ch(prometheus::core::MetricFamily::new(
                self.blocks_desc.clone(),
                prometheus::proto::MetricType::COUNTER,
                stats.blocks as f64,
                vec![device],
            ));
        }

        Ok(())
    }
}

impl Collector for DevstatCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&self.bytes_desc);
        descs(&self.transfers_desc);
        descs(&self.blocks_desc);
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        let mut ch = |metric: MetricFamily| {
            metrics(Box::new(metric));
        };
        if let Err(e) = self.update(&mut ch) {
            self.logger.error("failed to collect device stats", o!("error" => e.to_string()));
        }
    }
}