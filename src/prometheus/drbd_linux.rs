use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use slog::Logger;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::sync::Arc;

struct DrbdNumericalMetric {
    desc: Desc,
    value_type: prometheus::proto::MetricType,
    multiplier: f64,
}

impl DrbdNumericalMetric {
    fn new(name: &str, desc: &str, value_type: prometheus::proto::MetricType, multiplier: f64) -> Self {
        Self {
            desc: Desc::new(prometheus::core::build_fq_name("namespace", "drbd", name), desc, vec!["device"], None).unwrap(),
            value_type,
            multiplier,
        }
    }
}

struct DrbdStringPairMetric {
    desc: Desc,
    value_ok: String,
}

impl DrbdStringPairMetric {
    fn new(name: &str, desc: &str, value_ok: &str) -> Self {
        Self {
            desc: Desc::new(prometheus::core::build_fq_name("namespace", "drbd", name), desc, vec!["device", "node"], None).unwrap(),
            value_ok: value_ok.to_string(),
        }
    }

    fn is_okay(&self, v: &str) -> f64 {
        if v == self.value_ok {
            1.0
        } else {
            0.0
        }
    }
}

struct DrbdCollector {
    numerical: HashMap<String, DrbdNumericalMetric>,
    string_pair: HashMap<String, DrbdStringPairMetric>,
    connected: Desc,
    logger: Logger,
}

impl DrbdCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn std::error::Error>> {
        let numerical = vec![
            ("ns", DrbdNumericalMetric::new("network_sent_bytes_total", "Total number of bytes sent via the network.", prometheus::proto::MetricType::COUNTER, 1024.0)),
            ("nr", DrbdNumericalMetric::new("network_received_bytes_total", "Total number of bytes received via the network.", prometheus::proto::MetricType::COUNTER, 1.0)),
            ("dw", DrbdNumericalMetric::new("disk_written_bytes_total", "Net data written on local hard disk; in bytes.", prometheus::proto::MetricType::COUNTER, 1024.0)),
            ("dr", DrbdNumericalMetric::new("disk_read_bytes_total", "Net data read from local hard disk; in bytes.", prometheus::proto::MetricType::COUNTER, 1024.0)),
            ("al", DrbdNumericalMetric::new("activitylog_writes_total", "Number of updates of the activity log area of the meta data.", prometheus::proto::MetricType::COUNTER, 1.0)),
            ("bm", DrbdNumericalMetric::new("bitmap_writes_total", "Number of updates of the bitmap area of the meta data.", prometheus::proto::MetricType::COUNTER, 1.0)),
            ("lo", DrbdNumericalMetric::new("local_pending", "Number of open requests to the local I/O sub-system.", prometheus::proto::MetricType::GAUGE, 1.0)),
            ("pe", DrbdNumericalMetric::new("remote_pending", "Number of requests sent to the peer, but that have not yet been answered by the latter.", prometheus::proto::MetricType::GAUGE, 1.0)),
            ("ua", DrbdNumericalMetric::new("remote_unacknowledged", "Number of requests received by the peer via the network connection, but that have not yet been answered.", prometheus::proto::MetricType::GAUGE, 1.0)),
            ("ap", DrbdNumericalMetric::new("application_pending", "Number of block I/O requests forwarded to DRBD, but not yet answered by DRBD.", prometheus::proto::MetricType::GAUGE, 1.0)),
            ("ep", DrbdNumericalMetric::new("epochs", "Number of Epochs currently on the fly.", prometheus::proto::MetricType::GAUGE, 1.0)),
            ("oos", DrbdNumericalMetric::new("out_of_sync_bytes", "Amount of data known to be out of sync; in bytes.", prometheus::proto::MetricType::GAUGE, 1024.0)),
        ].into_iter().map(|(k, v)| (k.to_string(), v)).collect();

        let string_pair = vec![
            ("ro", DrbdStringPairMetric::new("node_role_is_primary", "Whether the role of the node is in the primary state.", "Primary")),
            ("ds", DrbdStringPairMetric::new("disk_state_is_up_to_date", "Whether the disk of the node is up to date.", "UpToDate")),
        ].into_iter().map(|(k, v)| (k.to_string(), v)).collect();

        Ok(Self {
            numerical,
            string_pair,
            connected: Desc::new(prometheus::core::build_fq_name("namespace", "drbd", "connected"), "Whether DRBD is connected to the peer.", vec!["device"], None).unwrap(),
            logger,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(MetricFamily)) -> Result<(), Box<dyn std::error::Error>> {
        let stats_file = "/proc/drbd";
        let file = File::open(stats_file)?;
        let reader = io::BufReader::new(file);
        let mut device = "unknown".to_string();

        for line in reader.lines() {
            let line = line?;
            let field = line.split_whitespace().collect::<Vec<&str>>();

            if field.len() != 2 {
                self.logger.debug("skipping invalid key:value pair", o!("field" => line));
                continue;
            }

            if let Ok(id) = field[0].parse::<u64>() {
                if field[1].is_empty() {
                    device = format!("drbd{}", id);
                    continue;
                }
            }

            if let Some(m) = self.numerical.get(field[0]) {
                let value = field[1].parse::<f64>()?;
                ch(must_new_const_metric(m.desc.clone(), m.value_type, value * m.multiplier, vec![device.clone()]));
                continue;
            }

            if let Some(m) = self.string_pair.get(field[0]) {
                let values = field[1].split('/').collect::<Vec<&str>>();
                ch(must_new_const_metric(m.desc.clone(), prometheus::proto::MetricType::GAUGE, m.is_okay(values[0]), vec![device.clone(), "local".to_string()]));
                ch(must_new_const_metric(m.desc.clone(), prometheus::proto::MetricType::GAUGE, m.is_okay(values[1]), vec![device.clone(), "remote".to_string()]));
                continue;
            }

            if field[0] == "cs" {
                let connected = if field[1] == "Connected" { 1.0 } else { 0.0 };
                ch(must_new_const_metric(self.connected.clone(), prometheus::proto::MetricType::GAUGE, connected, vec![device.clone()]));
                continue;
            }

            self.logger.debug("unhandled key-value pair", o!("key" => field[0], "value" => field[1]));
        }

        Ok(())
    }
}

impl Collector for DrbdCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        for metric in self.numerical.values() {
            descs(&metric.desc);
        }
        for metric in self.string_pair.values() {
            descs(&metric.desc);
        }
        descs(&self.connected);
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        let mut ch = |metric: MetricFamily| {
            metrics(Box::new(metric));
        };
        if let Err(e) = self.update(&mut ch) {
            self.logger.error("failed to collect DRBD metrics", o!("error" => e.to_string()));
        }
    }
}