use prometheus::{self, core::Desc};
use slog::Logger;
use regex::Regex;
use lazy_static::lazy_static;

const DISK_SUBSYSTEM: &str = "disk";

lazy_static! {
    static ref READS_COMPLETED_DESC: Desc = Desc::new(
        prometheus::core::build_fq_name("namespace", DISK_SUBSYSTEM, "reads_completed_total"),
        "The total number of reads completed successfully.",
        vec!["device".to_string()],
        None,
    ).unwrap();
    static ref READ_BYTES_DESC: Desc = Desc::new(
        prometheus::core::build_fq_name("namespace", DISK_SUBSYSTEM, "read_bytes_total"),
        "The total number of bytes read successfully.",
        vec!["device".to_string()],
        None,
    ).unwrap();
    static ref WRITES_COMPLETED_DESC: Desc = Desc::new(
        prometheus::core::build_fq_name("namespace", DISK_SUBSYSTEM, "writes_completed_total"),
        "The total number of writes completed successfully.",
        vec!["device".to_string()],
        None,
    ).unwrap();
    static ref WRITTEN_BYTES_DESC: Desc = Desc::new(
        prometheus::core::build_fq_name("namespace", DISK_SUBSYSTEM, "written_bytes_total"),
        "The total number of bytes written successfully.",
        vec!["device".to_string()],
        None,
    ).unwrap();
    static ref IO_TIME_SECONDS_DESC: Desc = Desc::new(
        prometheus::core::build_fq_name("namespace", DISK_SUBSYSTEM, "io_time_seconds_total"),
        "Total seconds spent doing I/Os.",
        vec!["device".to_string()],
        None,
    ).unwrap();
    static ref READ_TIME_SECONDS_DESC: Desc = Desc::new(
        prometheus::core::build_fq_name("namespace", DISK_SUBSYSTEM, "read_time_seconds_total"),
        "The total number of seconds spent by all reads.",
        vec!["device".to_string()],
        None,
    ).unwrap();
    static ref WRITE_TIME_SECONDS_DESC: Desc = Desc::new(
        prometheus::core::build_fq_name("namespace", DISK_SUBSYSTEM, "write_time_seconds_total"),
        "This is the total number of seconds spent by all writes.",
        vec!["device".to_string()],
        None,
    ).unwrap();
}

struct DeviceFilter {
    ignore_pattern: Option<Regex>,
    accept_pattern: Option<Regex>,
}

impl DeviceFilter {
    fn new(ignored_pattern: &str, accept_pattern: &str) -> Self {
        let ignore_pattern = if !ignored_pattern.is_empty() {
            Some(Regex::new(ignored_pattern).unwrap())
        } else {
            None
        };

        let accept_pattern = if !accept_pattern.is_empty() {
            Some(Regex::new(accept_pattern).unwrap())
        } else {
            None
        };

        DeviceFilter {
            ignore_pattern,
            accept_pattern,
        }
    }

    fn ignored(&self, name: &str) -> bool {
        (self.ignore_pattern.as_ref().map_or(false, |p| p.is_match(name)))
            || (self.accept_pattern.as_ref().map_or(false, |p| !p.is_match(name)))
    }
}

fn new_diskstats_device_filter(logger: &Logger) -> Result<DeviceFilter, Box<dyn std::error::Error>> {
    let old_diskstats_device_exclude = std::env::var("OLD_DISKSTATS_DEVICE_EXCLUDE").unwrap_or_default();
    let diskstats_device_exclude = std::env::var("DISKSTATS_DEVICE_EXCLUDE").unwrap_or_default();
    let diskstats_device_include = std::env::var("DISKSTATS_DEVICE_INCLUDE").unwrap_or_default();

    if !old_diskstats_device_exclude.is_empty() {
        if diskstats_device_exclude.is_empty() {
            logger.warn("--collector.diskstats.ignored-devices is DEPRECATED and will be removed in 2.0.0, use --collector.diskstats.device-exclude");
        } else {
            return Err("--collector.diskstats.ignored-devices and --collector.diskstats.device-exclude are mutually exclusive".into());
        }
    }

    if !diskstats_device_exclude.is_empty() && !diskstats_device_include.is_empty() {
        return Err("device-exclude & device-include are mutually exclusive".into());
    }

    if !diskstats_device_exclude.is_empty() {
        logger.info("Parsed flag --collector.diskstats.device-exclude", o!("flag" => diskstats_device_exclude.clone()));
    }

    if !diskstats_device_include.is_empty() {
        logger.info("Parsed flag --collector.diskstats.device-include", o!("flag" => diskstats_device_include.clone()));
    }

    Ok(DeviceFilter::new(&diskstats_device_exclude, &diskstats_device_include))
}