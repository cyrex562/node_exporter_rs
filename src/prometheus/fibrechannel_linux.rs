use prometheus::{self, core::{Collector, Desc, Metric, Opts}};
use slog::Logger;
use std::collections::HashMap;
use std::sync::Arc;

const MAX_UINT64: u64 = u64::MAX;

struct FibreChannelCollector {
    fs: SysFs,
    metric_descs: HashMap<String, Desc>,
    logger: Arc<Logger>,
    subsystem: String,
}

impl FibreChannelCollector {
    fn new(logger: Arc<Logger>) -> Result<Self, Box<dyn std::error::Error>> {
        let fs = SysFs::new(sys_path)?;
        let subsystem = "fibrechannel".to_string();
        let descriptions = vec![
            ("dumped_frames_total", "Number of dumped frames"),
            ("loss_of_signal_total", "Number of times signal has been lost"),
            ("loss_of_sync_total", "Number of failures on either bit or transmission word boundaries"),
            ("rx_frames_total", "Number of frames received"),
            ("error_frames_total", "Number of errors in frames"),
            ("invalid_tx_words_total", "Number of invalid words transmitted by host port"),
            ("seconds_since_last_reset_total", "Number of seconds since last host port reset"),
            ("tx_words_total", "Number of words transmitted by host port"),
            ("invalid_crc_total", "Invalid Cyclic Redundancy Check count"),
            ("nos_total", "Number Not_Operational Primitive Sequence received by host port"),
            ("fcp_packet_aborts_total", "Number of aborted packets"),
            ("rx_words_total", "Number of words received by host port"),
            ("tx_frames_total", "Number of frames transmitted by host port"),
            ("link_failure_total", "Number of times the host port link has failed"),
        ];

        let metric_descs = descriptions.into_iter().map(|(name, desc)| {
            let desc = Desc::new(
                Opts::new(name, desc)
                    .namespace(namespace)
                    .subsystem(&subsystem)
                    .variable_labels(vec!["fc_host".to_string()]),
            );
            (name.to_string(), desc)
        }).collect();

        Ok(FibreChannelCollector { fs, metric_descs, logger, subsystem })
    }

    fn push_metric(&self, ch: &mut dyn Collector, name: &str, value: u64, host: &str, value_type: prometheus::proto::MetricType) {
        if let Some(desc) = self.metric_descs.get(name) {
            ch.collect(Box::new(prometheus::Gauge::new(
                desc,
                value_type,
                value as f64,
                &[host.to_string()],
            )));
        }
    }

    fn push_counter(&self, ch: &mut dyn Collector, name: &str, value: u64, host: &str) {
        if value != MAX_UINT64 {
            self.push_metric(ch, name, value, host, prometheus::proto::MetricType::COUNTER);
        }
    }
}

impl Collector for FibreChannelCollector {
    fn describe(&self, descs: &mut Vec<&Desc>) {
        for desc in self.metric_descs.values() {
            descs.push(desc);
        }
    }

    fn collect(&self, mfs: &mut Vec<MetricFamily>) {
        if let Ok(hosts) = self.fs.fibre_channel_class() {
            for host in hosts {
                let info_desc = Desc::new(
                    Opts::new("info", "Non-numeric data from /sys/class/fc_host/<host>, value is always 1.")
                        .namespace(namespace)
                        .subsystem(&self.subsystem)
                        .variable_labels(vec![
                            "fc_host".to_string(), "speed".to_string(), "port_state".to_string(), "port_type".to_string(),
                            "port_id".to_string(), "port_name".to_string(), "fabric_name".to_string(), "symbolic_name".to_string(),
                            "supported_classes".to_string(), "supported_speeds".to_string(), "dev_loss_tmo".to_string(),
                        ]),
                );

                let info_value = 1.0;
                let labels = vec![
                    host.name.clone(), host.speed.clone(), host.port_state.clone(), host.port_type.clone(),
                    host.port_id.clone(), host.port_name.clone(), host.fabric_name.clone(), host.symbolic_name.clone(),
                    host.supported_classes.clone(), host.supported_speeds.clone(), host.dev_loss_tmo.clone(),
                ];

                mfs.push(prometheus::new_const_metric(&info_desc, prometheus::proto::MetricType::GAUGE, info_value, &labels));

                self.push_counter(mfs, "dumped_frames_total", host.counters.dumped_frames, &host.name);
                self.push_counter(mfs, "error_frames_total", host.counters.error_frames, &host.name);
                self.push_counter(mfs, "invalid_crc_total", host.counters.invalid_crc_count, &host.name);
                self.push_counter(mfs, "rx_frames_total", host.counters.rx_frames, &host.name);
                self.push_counter(mfs, "rx_words_total", host.counters.rx_words, &host.name);
                self.push_counter(mfs, "tx_frames_total", host.counters.tx_frames, &host.name);
                self.push_counter(mfs, "tx_words_total", host.counters.tx_words, &host.name);
                self.push_counter(mfs, "seconds_since_last_reset_total", host.counters.seconds_since_last_reset, &host.name);
                self.push_counter(mfs, "invalid_tx_words_total", host.counters.invalid_tx_word_count, &host.name);
                self.push_counter(mfs, "link_failure_total", host.counters.link_failure_count, &host.name);
                self.push_counter(mfs, "loss_of_sync_total", host.counters.loss_of_sync_count, &host.name);
                self.push_counter(mfs, "loss_of_signal_total", host.counters.loss_of_signal_count, &host.name);
                self.push_counter(mfs, "nos_total", host.counters.nos_count, &host.name);
                self.push_counter(mfs, "fcp_packet_aborts_total", host.counters.fcp_packet_aborts, &host.name);
            }
        } else {
            self.logger.debug("fibrechannel statistics not found, skipping");
        }
    }
}