use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::Arc;
use blockdevice::BlockDevice;

const SECONDS_PER_TICK: f64 = 1.0 / 1000.0;
const UNIX_SECTOR_SIZE: f64 = 512.0;
const DISKSTATS_DEFAULT_IGNORED_DEVICES: &str = "^(z?ram|loop|fd|(h|s|v|xv)d[a-z]|nvme\\d+n\\d+p)\\d+$";

struct TypedFactorDesc {
    desc: Desc,
    value_type: prometheus::proto::MetricType,
}

impl TypedFactorDesc {
    fn new(name: &str, help: &str, labels: Vec<&str>, value_type: prometheus::proto::MetricType) -> Self {
        Self {
            desc: Desc::new(prometheus::core::build_fq_name("namespace", "disk", name), help, labels, None).unwrap(),
            value_type,
        }
    }

    fn must_new_const_metric(&self, value: f64, labels: Vec<&str>) -> prometheus::proto::Metric {
        prometheus::proto::Metric::new(self.desc.clone(), self.value_type, value, labels)
    }
}

struct DiskstatsCollector {
    device_filter: DeviceFilter,
    fs: BlockDevice,
    info_desc: TypedFactorDesc,
    descs: Vec<TypedFactorDesc>,
    filesystem_info_desc: TypedFactorDesc,
    device_mapper_info_desc: TypedFactorDesc,
    ata_descs: HashMap<String, TypedFactorDesc>,
    logger: Logger,
    get_udev_device_properties: Option<fn(u32, u32) -> Result<UdevInfo, io::Error>>,
}

impl DiskstatsCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        let disk_label_names = vec!["device"];
        let fs = BlockDevice::new("/proc", "/sys")?;
        let device_filter = new_diskstats_device_filter(&logger)?;

        let mut collector = DiskstatsCollector {
            device_filter,
            fs,
            info_desc: TypedFactorDesc::new(
                "info",
                "Info of /sys/block/<block_device>.",
                vec!["device", "major", "minor", "path", "wwn", "model", "serial", "revision"],
                prometheus::proto::MetricType::GAUGE,
            ),
            descs: vec![
                TypedFactorDesc::new("reads_completed_total", "The total number of reads completed successfully.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                TypedFactorDesc::new("reads_merged_total", "The total number of reads merged.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                TypedFactorDesc::new("read_bytes_total", "The total number of bytes read successfully.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                TypedFactorDesc::new("read_time_seconds_total", "The total number of seconds spent by all reads.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                TypedFactorDesc::new("writes_completed_total", "The total number of writes completed successfully.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                TypedFactorDesc::new("writes_merged_total", "The number of writes merged.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                TypedFactorDesc::new("written_bytes_total", "The total number of bytes written successfully.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                TypedFactorDesc::new("write_time_seconds_total", "The total number of seconds spent by all writes.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                TypedFactorDesc::new("io_now", "The number of I/Os currently in progress.", disk_label_names.clone(), prometheus::proto::MetricType::GAUGE),
                TypedFactorDesc::new("io_time_seconds_total", "Total seconds spent doing I/Os.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                TypedFactorDesc::new("io_time_weighted_seconds_total", "The weighted # of seconds spent doing I/Os.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                TypedFactorDesc::new("discards_completed_total", "The total number of discards completed successfully.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                TypedFactorDesc::new("discards_merged_total", "The total number of discards merged.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                TypedFactorDesc::new("discarded_sectors_total", "The total number of sectors discarded successfully.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                TypedFactorDesc::new("discard_time_seconds_total", "This is the total number of seconds spent by all discards.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                TypedFactorDesc::new("flush_requests_total", "The total number of flush requests completed successfully", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
                TypedFactorDesc::new("flush_requests_time_seconds_total", "This is the total number of seconds spent by all flush requests.", disk_label_names.clone(), prometheus::proto::MetricType::COUNTER),
            ],
            filesystem_info_desc: TypedFactorDesc::new(
                "filesystem_info",
                "Info about disk filesystem.",
                vec!["device", "type", "usage", "uuid", "version"],
                prometheus::proto::MetricType::GAUGE,
            ),
            device_mapper_info_desc: TypedFactorDesc::new(
                "device_mapper_info",
                "Info about disk device mapper.",
                vec!["device", "name", "uuid", "vg_name", "lv_name", "lv_layer"],
                prometheus::proto::MetricType::GAUGE,
            ),
            ata_descs: HashMap::new(),
            logger,
            get_udev_device_properties: None,
        };

        collector.ata_descs.insert(
            "ID_ATA_WRITE_CACHE".to_string(),
            TypedFactorDesc::new(
                "ata_write_cache",
                "ATA disk has a write cache.",
                vec!["device"],
                prometheus::proto::MetricType::GAUGE,
            ),
        );
        collector.ata_descs.insert(
            "ID_ATA_WRITE_CACHE_ENABLED".to_string(),
            TypedFactorDesc::new(
                "ata_write_cache_enabled",
                "ATA disk has its write cache enabled.",
                vec!["device"],
                prometheus::proto::MetricType::GAUGE,
            ),
        );
        collector.ata_descs.insert(
            "ID_ATA_ROTATION_RATE_RPM".to_string(),
            TypedFactorDesc::new(
                "ata_rotation_rate_rpm",
                "ATA disk rotation rate in RPMs (0 for SSDs).",
                vec!["device"],
                prometheus::proto::MetricType::GAUGE,
            ),
        );

        if let Ok(stat) = std::fs::metadata("/run/udev/data") {
            if stat.is_dir() {
                collector.get_udev_device_properties = Some(get_udev_device_properties);
            } else {
                logger.error("Failed to open directory, disabling udev device properties", o!("path" => "/run/udev/data"));
            }
        }

        Ok(collector)
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let disk_stats = self.fs.proc_diskstats()?;

        for stats in disk_stats {
            let dev = &stats.device_name;
            if self.device_filter.ignored(dev) {
                continue;
            }

            let info = match self.get_udev_device_properties {
                Some(get_props) => get_props(stats.major_number, stats.minor_number).unwrap_or_default(),
                None => HashMap::new(),
            };

            let serial = info.get("SCSI_IDENT_SERIAL").or_else(|| info.get("ID_SERIAL_SHORT")).unwrap_or(&"".to_string());

            ch(self.info_desc.must_new_const_metric(1.0, vec![
                dev,
                &stats.major_number.to_string(),
                &stats.minor_number.to_string(),
                info.get("ID_PATH").unwrap_or(&"".to_string()),
                info.get("ID_WWN").unwrap_or(&"".to_string()),
                info.get("ID_MODEL").unwrap_or(&"".to_string()),
                serial,
                info.get("ID_REVISION").unwrap_or(&"".to_string()),
            ]));

            let stat_count = stats.io_stats_count - 3;

            for (i, val) in vec![
                stats.read_ios as f64,
                stats.read_merges as f64,
                stats.read_sectors as f64 * UNIX_SECTOR_SIZE,
                stats.read_ticks as f64 * SECONDS_PER_TICK,
                stats.write_ios as f64,
                stats.write_merges as f64,
                stats.write_sectors as f64 * UNIX_SECTOR_SIZE,
                stats.write_ticks as f64 * SECONDS_PER_TICK,
                stats.ios_in_progress as f64,
                stats.io_ticks as f64 * SECONDS_PER_TICK,
                stats.weighted_io_ticks as f64 * SECONDS_PER_TICK,
                stats.discard_ios as f64,
                stats.discard_merges as f64,
                stats.discard_sectors as f64,
                stats.discard_ticks as f64 * SECONDS_PER_TICK,
                stats.flush_requests_completed as f64,
                stats.time_spent_flushing as f64 * SECONDS_PER_TICK,
            ].into_iter().enumerate() {
                if i >= stat_count as usize {
                    break;
                }
                ch(self.descs[i].must_new_const_metric(val, vec![dev]));
            }

            if let Some(fs_type) = info.get("ID_FS_TYPE") {
                ch(self.filesystem_info_desc.must_new_const_metric(1.0, vec![
                    dev,
                    fs_type,
                    info.get("ID_FS_USAGE").unwrap_or(&"".to_string()),
                    info.get("ID_FS_UUID").unwrap_or(&"".to_string()),
                    info.get("ID_FS_VERSION").unwrap_or(&"".to_string()),
                ]));
            }

            if let Some(name) = info.get("DM_NAME") {
                ch(self.device_mapper_info_desc.must_new_const_metric(1.0, vec![
                    dev,
                    name,
                    info.get("DM_UUID").unwrap_or(&"".to_string()),
                    info.get("DM_VG_NAME").unwrap_or(&"".to_string()),
                    info.get("DM_LV_NAME").unwrap_or(&"".to_string()),
                    info.get("DM_LV_LAYER").unwrap_or(&"".to_string()),
                ]));
            }

            if info.contains_key("ID_ATA") {
                for (attr, desc) in &self.ata_descs {
                    if let Some(str_val) = info.get(attr) {
                        if let Ok(value) = str_val.parse::<f64>() {
                            ch(desc.must_new_const_metric(value, vec![dev]));
                        } else {
                            self.logger.error("Failed to parse ATA value", o!("err" => str_val));
                        }
                    } else {
                        self.logger.debug("Udev attribute does not exist", o!("attribute" => attr));
                    }
                }
            }
        }
        Ok(())
    }
}

impl Collector for DiskstatsCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&self.info_desc.desc);
        for desc in &self.descs {
            descs(&desc.desc);
        }
        descs(&self.filesystem_info_desc.desc);
        descs(&self.device_mapper_info_desc.desc);
        for desc in self.ata_descs.values() {
            descs(&desc.desc);
        }
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        let mut ch = |metric: MetricFamily| {
            metrics(Box::new(metric));
        };
        if let Err(e) = self.update(&mut ch) {
            self.logger.error("failed to collect disk stats", o!("error" => e.to_string()));
        }
    }
}

fn get_udev_device_properties(major: u32, minor: u32) -> Result<HashMap<String, String>, io::Error> {
    let filename = format!("/run/udev/data/b{}:{}", major, minor);
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);

    let mut info = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        if !line.starts_with("E:") {
            continue;
        }
        let line = &line[2..];
        if let Some((name, value)) = line.split_once('=') {
            info.insert(name.to_string(), value.to_string());
        }
    }
    Ok(info)
}