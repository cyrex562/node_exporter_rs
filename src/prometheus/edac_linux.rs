use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use regex::Regex;

const EDAC_SUBSYSTEM: &str = "edac";

lazy_static! {
    static ref EDAC_MEM_CONTROLLER_RE: Regex = Regex::new(r".*devices/system/edac/mc/mc([0-9]*)").unwrap();
    static ref EDAC_MEM_CSROW_RE: Regex = Regex::new(r".*devices/system/edac/mc/mc[0-9]*/csrow([0-9]*)").unwrap();
}

struct EdacCollector {
    ce_count: Desc,
    ue_count: Desc,
    cs_row_ce_count: Desc,
    cs_row_ue_count: Desc,
    logger: Logger,
}

impl EdacCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            ce_count: Desc::new(
                prometheus::core::build_fq_name("namespace", EDAC_SUBSYSTEM, "correctable_errors_total"),
                "Total correctable memory errors.",
                vec!["controller".to_string()],
                None,
            )?,
            ue_count: Desc::new(
                prometheus::core::build_fq_name("namespace", EDAC_SUBSYSTEM, "uncorrectable_errors_total"),
                "Total uncorrectable memory errors.",
                vec!["controller".to_string()],
                None,
            )?,
            cs_row_ce_count: Desc::new(
                prometheus::core::build_fq_name("namespace", EDAC_SUBSYSTEM, "csrow_correctable_errors_total"),
                "Total correctable memory errors for this csrow.",
                vec!["controller".to_string(), "csrow".to_string()],
                None,
            )?,
            cs_row_ue_count: Desc::new(
                prometheus::core::build_fq_name("namespace", EDAC_SUBSYSTEM, "csrow_uncorrectable_errors_total"),
                "Total uncorrectable memory errors for this csrow.",
                vec!["controller".to_string(), "csrow".to_string()],
                None,
            )?,
            logger,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let mem_controllers = glob::glob("/sys/devices/system/edac/mc/mc[0-9]*")?;
        for controller in mem_controllers {
            let controller = controller?;
            let controller_str = controller.to_str().unwrap();
            let controller_match = EDAC_MEM_CONTROLLER_RE.captures(controller_str).ok_or_else(|| format!("controller string didn't match regexp: {}", controller_str))?;
            let controller_number = &controller_match[1];

            let value = read_uint_from_file(controller.join("ce_count"))?;
            ch(prometheus::core::MetricFamily::new(
                self.ce_count.clone(),
                prometheus::proto::MetricType::COUNTER,
                value as f64,
                vec![controller_number.to_string()],
            ));

            let value = read_uint_from_file(controller.join("ce_noinfo_count"))?;
            ch(prometheus::core::MetricFamily::new(
                self.cs_row_ce_count.clone(),
                prometheus::proto::MetricType::COUNTER,
                value as f64,
                vec![controller_number.to_string(), "unknown".to_string()],
            ));

            let value = read_uint_from_file(controller.join("ue_count"))?;
            ch(prometheus::core::MetricFamily::new(
                self.ue_count.clone(),
                prometheus::proto::MetricType::COUNTER,
                value as f64,
                vec![controller_number.to_string()],
            ));

            let value = read_uint_from_file(controller.join("ue_noinfo_count"))?;
            ch(prometheus::core::MetricFamily::new(
                self.cs_row_ue_count.clone(),
                prometheus::proto::MetricType::COUNTER,
                value as f64,
                vec![controller_number.to_string(), "unknown".to_string()],
            ));

            let csrows = glob::glob(&format!("{}/csrow[0-9]*", controller_str))?;
            for csrow in csrows {
                let csrow = csrow?;
                let csrow_str = csrow.to_str().unwrap();
                let csrow_match = EDAC_MEM_CSROW_RE.captures(csrow_str).ok_or_else(|| format!("csrow string didn't match regexp: {}", csrow_str))?;
                let csrow_number = &csrow_match[1];

                let value = read_uint_from_file(csrow.join("ce_count"))?;
                ch(prometheus::core::MetricFamily::new(
                    self.cs_row_ce_count.clone(),
                    prometheus::proto::MetricType::COUNTER,
                    value as f64,
                    vec![controller_number.to_string(), csrow_number.to_string()],
                ));

                let value = read_uint_from_file(csrow.join("ue_count"))?;
                ch(prometheus::core::MetricFamily::new(
                    self.cs_row_ue_count.clone(),
                    prometheus::proto::MetricType::COUNTER,
                    value as f64,
                    vec![controller_number.to_string(), csrow_number.to_string()],
                ));
            }
        }
        Ok(())
    }
}

impl Collector for EdacCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&self.ce_count);
        descs(&self.ue_count);
        descs(&self.cs_row_ce_count);
        descs(&self.cs_row_ue_count);
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        let mut ch = |metric: MetricFamily| {
            metrics(Box::new(metric));
        };
        if let Err(e) = self.update(&mut ch) {
            self.logger.error("failed to collect EDAC metrics", o!("error" => e.to_string()));
        }
    }
}

fn read_uint_from_file<P: AsRef<Path>>(path: P) -> Result<u64, std::io::Error> {
    let content = fs::read_to_string(path)?;
    content.trim().parse().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}