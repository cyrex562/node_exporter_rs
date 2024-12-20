use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::sync::Arc;

struct DmiCollector {
    info_desc: Desc,
    values: Vec<String>,
}

impl DmiCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        let sys_path = "/sys/class/dmi/id";
        let dmi = match fs::read_dir(sys_path) {
            Ok(entries) => {
                let mut dmi = HashMap::new();
                for entry in entries {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(value) = fs::read_to_string(&path) {
                            dmi.insert(entry.file_name().into_string().unwrap(), value.trim().to_string());
                        }
                    }
                }
                dmi
            }
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    logger.debug("Platform does not support Desktop Management Interface (DMI) information", o!("err" => err.to_string()));
                    HashMap::new()
                } else {
                    return Err(format!("failed to read Desktop Management Interface (DMI) information: {}", err).into());
                }
            }
        };

        let mut labels = Vec::new();
        let mut values = Vec::new();
        for (label, value) in &dmi {
            labels.push(label.clone());
            values.push(value.clone());
        }

        Ok(Self {
            info_desc: Desc::new(
                prometheus::core::build_fq_name("namespace", "dmi", "info"),
                "A metric with a constant '1' value labeled by bios_date, bios_release, bios_vendor, bios_version, \
                board_asset_tag, board_name, board_serial, board_vendor, board_version, chassis_asset_tag, \
                chassis_serial, chassis_vendor, chassis_version, product_family, product_name, product_serial, \
                product_sku, product_uuid, product_version, system_vendor if provided by DMI.",
                labels,
                None,
            )?,
            values,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        if self.values.is_empty() {
            return Err("No data".into());
        }
        ch(prometheus::core::MetricFamily::new(
            self.info_desc.clone(),
            prometheus::proto::MetricType::GAUGE,
            1.0,
            self.values.clone(),
        ));
        Ok(())
    }
}

impl Collector for DmiCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&self.info_desc);
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        let mut ch = |metric: MetricFamily| {
            metrics(Box::new(metric));
        };
        if let Err(e) = self.update(&mut ch) {
            eprintln!("failed to collect DMI metrics: {}", e);
        }
    }
}