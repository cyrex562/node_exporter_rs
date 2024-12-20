use clap::{Arg, App};
use regex::Regex;

lazy_static! {
    static ref ETHTOOL_DEVICE_INCLUDE: String = App::new("ethtool")
        .arg(Arg::with_name("collector.ethtool.device-include")
            .help("Regexp of ethtool devices to include (mutually exclusive to device-exclude)")
            .takes_value(true))
        .get_matches()
        .value_of("collector.ethtool.device-include")
        .unwrap_or("")
        .to_string();

    static ref ETHTOOL_DEVICE_EXCLUDE: String = App::new("ethtool")
        .arg(Arg::with_name("collector.ethtool.device-exclude")
            .help("Regexp of ethtool devices to exclude (mutually exclusive to device-include)")
            .takes_value(true))
        .get_matches()
        .value_of("collector.ethtool.device-exclude")
        .unwrap_or("")
        .to_string();

    static ref ETHTOOL_INCLUDED_METRICS: String = App::new("ethtool")
        .arg(Arg::with_name("collector.ethtool.metrics-include")
            .help("Regexp of ethtool stats to include.")
            .default_value(".*")
            .takes_value(true))
        .get_matches()
        .value_of("collector.ethtool.metrics-include")
        .unwrap_or(".*")
        .to_string();

    static ref ETHTOOL_RECEIVED_REGEX: Regex = Regex::new(r"(^|_)rx(_|$)").unwrap();
    static ref ETHTOOL_TRANSMIT_REGEX: Regex = Regex::new(r"(^|_)tx(_|$)").unwrap();
}

use std::collections::HashMap;
use regex::Regex;
use prometheus::core::Desc;
use slog::Logger;
use std::sync::Mutex;

trait Ethtool {
    fn driver_info(&self, intf: &str) -> Result<DrvInfo, Box<dyn std::error::Error>>;
    fn stats(&self, intf: &str) -> Result<HashMap<String, u64>, Box<dyn std::error::Error>>;
    fn link_info(&self, intf: &str) -> Result<EthtoolCmd, Box<dyn std::error::Error>>;
}

struct EthtoolLibrary {
    ethtool: Ethtool,
}

impl Ethtool for EthtoolLibrary {
    fn driver_info(&self, intf: &str) -> Result<DrvInfo, Box<dyn std::error::Error>> {
        self.ethtool.driver_info(intf)
    }

    fn stats(&self, intf: &str) -> Result<HashMap<String, u64>, Box<dyn std::error::Error>> {
        self.ethtool.stats(intf)
    }

    fn link_info(&self, intf: &str) -> Result<EthtoolCmd, Box<dyn std::error::Error>> {
        let mut ethtool_cmd = EthtoolCmd::default();
        ethtool_cmd.cmd_get(intf)?;
        Ok(ethtool_cmd)
    }
}

struct EthtoolCollector {
    fs: SysFs,
    entries: HashMap<String, Desc>,
    entries_mutex: Mutex<()>,
    ethtool: Box<dyn Ethtool>,
    device_filter: DeviceFilter,
    info_desc: Desc,
    metrics_pattern: Regex,
    logger: Logger,
}

use std::collections::HashMap;
use prometheus::{Desc, Opts};
use regex::Regex;
use slog::Logger;

fn make_ethtool_collector(logger: &Logger) -> Result<EthtoolCollector, Box<dyn std::error::Error>> {
    let fs = SysFs::new(sys_path)?;
    let e = Ethtool::new()?;

    if !ETHTOOL_DEVICE_INCLUDE.is_empty() {
        logger.info("Parsed flag --collector.ethtool.device-include", o!("flag" => ETHTOOL_DEVICE_INCLUDE.clone()));
    }
    if !ETHTOOL_DEVICE_EXCLUDE.is_empty() {
        logger.info("Parsed flag --collector.ethtool.device-exclude", o!("flag" => ETHTOOL_DEVICE_EXCLUDE.clone()));
    }
    if !ETHTOOL_INCLUDED_METRICS.is_empty() {
        logger.info("Parsed flag --collector.ethtool.metrics-include", o!("flag" => ETHTOOL_INCLUDED_METRICS.clone()));
    }

    let entries = hashmap!{
        "rx_bytes".to_string() => Desc::new(
            Opts::new("received_bytes_total", "Network interface bytes received")
                .namespace("ethtool")
                .variable_labels(vec!["device".to_string()])
        )?,
        "rx_dropped".to_string() => Desc::new(
            Opts::new("received_dropped_total", "Number of received frames dropped")
                .namespace("ethtool")
                .variable_labels(vec!["device".to_string()])
        )?,
        "rx_errors".to_string() => Desc::new(
            Opts::new("received_errors_total", "Number of received frames with errors")
                .namespace("ethtool")
                .variable_labels(vec!["device".to_string()])
        )?,
        "rx_packets".to_string() => Desc::new(
            Opts::new("received_packets_total", "Network interface packets received")
                .namespace("ethtool")
                .variable_labels(vec!["device".to_string()])
        )?,
        "tx_bytes".to_string() => Desc::new(
            Opts::new("transmitted_bytes_total", "Network interface bytes sent")
                .namespace("ethtool")
                .variable_labels(vec!["device".to_string()])
        )?,
        "tx_errors".to_string() => Desc::new(
            Opts::new("transmitted_errors_total", "Number of sent frames with errors")
                .namespace("ethtool")
                .variable_labels(vec!["device".to_string()])
        )?,
        "tx_packets".to_string() => Desc::new(
            Opts::new("transmitted_packets_total", "Network interface packets sent")
                .namespace("ethtool")
                .variable_labels(vec!["device".to_string()])
        )?,
        "supported_port".to_string() => Desc::new(
            Opts::new("supported_port_info", "Type of ports or PHYs supported by network device")
                .namespace("network")
                .variable_labels(vec!["device".to_string(), "type".to_string()])
        )?,
        "supported_speed".to_string() => Desc::new(
            Opts::new("supported_speed_bytes", "Combination of speeds and features supported by network device")
                .namespace("network")
                .variable_labels(vec!["device".to_string(), "duplex".to_string(), "mode".to_string()])
        )?,
        "supported_autonegotiate".to_string() => Desc::new(
            Opts::new("autonegotiate_supported", "If this port device supports autonegotiate")
                .namespace("network")
                .variable_labels(vec!["device".to_string()])
        )?,
        "supported_pause".to_string() => Desc::new(
            Opts::new("pause_supported", "If this port device supports pause frames")
                .namespace("network")
                .variable_labels(vec!["device".to_string()])
        )?,
        "supported_asymmetricpause".to_string() => Desc::new(
            Opts::new("asymmetricpause_supported", "If this port device supports asymmetric pause frames")
                .namespace("network")
                .variable_labels(vec!["device".to_string()])
        )?,
        "advertised_speed".to_string() => Desc::new(
            Opts::new("advertised_speed_bytes", "Combination of speeds and features offered by network device")
                .namespace("network")
                .variable_labels(vec!["device".to_string(), "duplex".to_string(), "mode".to_string()])
        )?,
        "advertised_autonegotiate".to_string() => Desc::new(
            Opts::new("autonegotiate_advertised", "If this port device offers autonegotiate")
                .namespace("network")
                .variable_labels(vec!["device".to_string()])
        )?,
        "advertised_pause".to_string() => Desc::new(
            Opts::new("pause_advertised", "If this port device offers pause capability")
                .namespace("network")
                .variable_labels(vec!["device".to_string()])
        )?,
        "advertised_asymmetricpause".to_string() => Desc::new(
            Opts::new("asymmetricpause_advertised", "If this port device offers asymmetric pause capability")
                .namespace("network")
                .variable_labels(vec!["device".to_string()])
        )?,
        "autonegotiate".to_string() => Desc::new(
            Opts::new("autonegotiate", "If this port is using autonegotiate")
                .namespace("network")
                .variable_labels(vec!["device".to_string()])
        )?,
    };

    let info_desc = Desc::new(
        Opts::new("info", "A metric with a constant '1' value labeled by bus_info, device, driver, expansion_rom_version, firmware_version, version.")
            .namespace("ethtool")
            .variable_labels(vec!["bus_info".to_string(), "device".to_string(), "driver".to_string(), "expansion_rom_version".to_string(), "firmware_version".to_string(), "version".to_string()])
    )?;

    Ok(EthtoolCollector {
        fs,
        ethtool: Box::new(EthtoolLibrary { ethtool: e }),
        device_filter: DeviceFilter::new(&ETHTOOL_DEVICE_EXCLUDE, &ETHTOOL_DEVICE_INCLUDE),
        metrics_pattern: Regex::new(&ETHTOOL_INCLUDED_METRICS)?,
        logger: logger.clone(),
        entries,
        info_desc,
        entries_mutex: Mutex::new(()),
    })
}

use prometheus::{register_collector, Collector};
use regex::Regex;
use slog::Logger;

fn init() {
    register_collector("ethtool", default_disabled, new_ethtool_collector);
}

fn build_ethtool_fq_name(metric: &str) -> String {
    let metric_name = sanitize_metric_name(metric).to_lowercase().trim_start_matches('_').to_string();
    let metric_name = ETHTOOL_RECEIVED_REGEX.replace_all(&metric_name, "${1}received${2}").to_string();
    let metric_name = ETHTOOL_TRANSMIT_REGEX.replace_all(&metric_name, "${1}transmitted${2}").to_string();
    prometheus::build_fq_name(namespace, "ethtool", &metric_name)
}

fn new_ethtool_collector(logger: &Logger) -> Result<Box<dyn Collector>, Box<dyn std::error::Error>> {
    Ok(Box::new(make_ethtool_collector(logger)?))
}

use prometheus::{self, core::Collector, core::Desc, proto::MetricFamily};
use slog::Logger;
use std::collections::HashMap;
use std::sync::Mutex;

impl EthtoolCollector {
    fn update_port_capabilities(&self, ch: &mut dyn Collector, prefix: &str, device: &str, link_modes: u32) {
        let autonegotiate = if link_modes & (1 << unix::ETHTOOL_LINK_MODE_Autoneg_BIT) != 0 { 1.0 } else { 0.0 };
        let pause = if link_modes & (1 << unix::ETHTOOL_LINK_MODE_Pause_BIT) != 0 { 1.0 } else { 0.0 };
        let asymmetric_pause = if link_modes & (1 << unix::ETHTOOL_LINK_MODE_Asym_Pause_BIT) != 0 { 1.0 } else { 0.0 };

        ch.collect(Box::new(prometheus::Gauge::new(
            &format!("{}_autonegotiate", prefix),
            "Autonegotiate capability",
            &[device],
            autonegotiate,
        )));
        ch.collect(Box::new(prometheus::Gauge::new(
            &format!("{}_pause", prefix),
            "Pause capability",
            &[device],
            pause,
        )));
        ch.collect(Box::new(prometheus::Gauge::new(
            &format!("{}_asymmetricpause", prefix),
            "Asymmetric pause capability",
            &[device],
            asymmetric_pause,
        )));
    }

    fn update_port_info(&self, ch: &mut dyn Collector, device: &str, link_modes: u32) {
        let port_types = [
            ("TP", unix::ETHTOOL_LINK_MODE_TP_BIT),
            ("AUI", unix::ETHTOOL_LINK_MODE_AUI_BIT),
            ("MII", unix::ETHTOOL_LINK_MODE_MII_BIT),
            ("FIBRE", unix::ETHTOOL_LINK_MODE_FIBRE_BIT),
            ("BNC", unix::ETHTOOL_LINK_MODE_BNC_BIT),
            ("Backplane", unix::ETHTOOL_LINK_MODE_Backplane_BIT),
        ];

        for (name, bit) in &port_types {
            if link_modes & (1 << bit) != 0 {
                ch.collect(Box::new(prometheus::Gauge::new(
                    "supported_port",
                    "Supported port type",
                    &[device, name],
                    1.0,
                )));
            }
        }
    }

    impl EthtoolCollector {
        fn update_speeds(&self, ch: &mut dyn Collector, prefix: &str, device: &str, link_modes: u32) {
            const MBPS: f64 = 1000000.0 / 8.0;
            let link_mode = format!("{}_speed", prefix);
            let speeds = [
                (unix::ETHTOOL_LINK_MODE_10baseT_Half_BIT, 10, "half", "T"),
                (unix::ETHTOOL_LINK_MODE_10baseT_Full_BIT, 10, "full", "T"),
                (unix::ETHTOOL_LINK_MODE_100baseT_Half_BIT, 100, "half", "T"),
                (unix::ETHTOOL_LINK_MODE_100baseT_Full_BIT, 100, "full", "T"),
                (unix::ETHTOOL_LINK_MODE_1000baseT_Half_BIT, 1000, "half", "T"),
                (unix::ETHTOOL_LINK_MODE_1000baseT_Full_BIT, 1000, "full", "T"),
                (unix::ETHTOOL_LINK_MODE_10000baseT_Full_BIT, 10000, "full", "T"),
                (unix::ETHTOOL_LINK_MODE_2500baseT_Full_BIT, 2500, "full", "T"),
                (unix::ETHTOOL_LINK_MODE_1000baseKX_Full_BIT, 1000, "full", "KX"),
                (unix::ETHTOOL_LINK_MODE_10000baseKX4_Full_BIT, 10000, "full", "KX4"),
                (unix::ETHTOOL_LINK_MODE_10000baseKR_Full_BIT, 10000, "full", "KR"),
                (unix::ETHTOOL_LINK_MODE_10000baseR_FEC_BIT, 10000, "full", "R_FEC"),
                (unix::ETHTOOL_LINK_MODE_20000baseMLD2_Full_BIT, 20000, "full", "MLD2"),
                (unix::ETHTOOL_LINK_MODE_20000baseKR2_Full_BIT, 20000, "full", "KR2"),
                (unix::ETHTOOL_LINK_MODE_40000baseKR4_Full_BIT, 40000, "full", "KR4"),
                (unix::ETHTOOL_LINK_MODE_40000baseCR4_Full_BIT, 40000, "full", "CR4"),
                (unix::ETHTOOL_LINK_MODE_40000baseSR4_Full_BIT, 40000, "full", "SR4"),
                (unix::ETHTOOL_LINK_MODE_40000baseLR4_Full_BIT, 40000, "full", "LR4"),
                (unix::ETHTOOL_LINK_MODE_56000baseKR4_Full_BIT, 56000, "full", "KR4"),
                (unix::ETHTOOL_LINK_MODE_56000baseCR4_Full_BIT, 56000, "full", "CR4"),
                (unix::ETHTOOL_LINK_MODE_56000baseSR4_Full_BIT, 56000, "full", "SR4"),
                (unix::ETHTOOL_LINK_MODE_56000baseLR4_Full_BIT, 56000, "full", "LR4"),
                (unix::ETHTOOL_LINK_MODE_25000baseCR_Full_BIT, 25000, "full", "CR"),
                (unix::ETHTOOL_LINK_MODE_25000baseKR_Full_BIT, 25000, "full", "KR"),
                (unix::ETHTOOL_LINK_MODE_25000baseSR_Full_BIT, 25000, "full", "SR"),
                (unix::ETHTOOL_LINK_MODE_50000baseCR2_Full_BIT, 50000, "full", "CR2"),
                (unix::ETHTOOL_LINK_MODE_50000baseKR2_Full_BIT, 50000, "full", "KR2"),
                (unix::ETHTOOL_LINK_MODE_100000baseKR4_Full_BIT, 100000, "full", "KR4"),
                (unix::ETHTOOL_LINK_MODE_100000baseSR4_Full_BIT, 100000, "full", "SR4"),
                (unix::ETHTOOL_LINK_MODE_100000baseCR4_Full_BIT, 100000, "full", "CR4"),
                (unix::ETHTOOL_LINK_MODE_100000baseLR4_ER4_Full_BIT, 100000, "full", "R4_ER4"),
                (unix::ETHTOOL_LINK_MODE_50000baseSR2_Full_BIT, 50000, "full", "SR2"),
                (unix::ETHTOOL_LINK_MODE_1000baseX_Full_BIT, 1000, "full", "X"),
                (unix::ETHTOOL_LINK_MODE_10000baseCR_Full_BIT, 10000, "full", "CR"),
                (unix::ETHTOOL_LINK_MODE_10000baseSR_Full_BIT, 10000, "full", "SR"),
                (unix::ETHTOOL_LINK_MODE_10000baseLR_Full_BIT, 10000, "full", "LR"),
                (unix::ETHTOOL_LINK_MODE_10000baseLRM_Full_BIT, 10000, "full", "LRM"),
                (unix::ETHTOOL_LINK_MODE_10000baseER_Full_BIT, 10000, "full", "ER"),
                (unix::ETHTOOL_LINK_MODE_5000baseT_Full_BIT, 5000, "full", "T"),
                (unix::ETHTOOL_LINK_MODE_50000baseKR_Full_BIT, 50000, "full", "KR"),
                (unix::ETHTOOL_LINK_MODE_50000baseSR_Full_BIT, 50000, "full", "SR"),
                (unix::ETHTOOL_LINK_MODE_50000baseCR_Full_BIT, 50000, "full", "CR"),
                (unix::ETHTOOL_LINK_MODE_50000baseLR_ER_FR_Full_BIT, 50000, "full", "LR_ER_FR"),
                (unix::ETHTOOL_LINK_MODE_50000baseDR_Full_BIT, 50000, "full", "DR"),
                (unix::ETHTOOL_LINK_MODE_100000baseKR2_Full_BIT, 100000, "full", "KR2"),
                (unix::ETHTOOL_LINK_MODE_100000baseSR2_Full_BIT, 100000, "full", "SR2"),
                (unix::ETHTOOL_LINK_MODE_100000baseCR2_Full_BIT, 100000, "full", "CR2"),
                (unix::ETHTOOL_LINK_MODE_100000baseLR2_ER2_FR2_Full_BIT, 100000, "full", "LR2_ER2_FR2"),
                (unix::ETHTOOL_LINK_MODE_100000baseDR2_Full_BIT, 100000, "full", "DR2"),
                (unix::ETHTOOL_LINK_MODE_200000baseKR4_Full_BIT, 200000, "full", "KR4"),
                (unix::ETHTOOL_LINK_MODE_200000baseSR4_Full_BIT, 200000, "full", "SR4"),
                (unix::ETHTOOL_LINK_MODE_200000baseLR4_ER4_FR4_Full_BIT, 200000, "full", "LR4_ER4_FR4"),
                (unix::ETHTOOL_LINK_MODE_200000baseDR4_Full_BIT, 200000, "full", "DR4"),
                (unix::ETHTOOL_LINK_MODE_200000baseCR4_Full_BIT, 200000, "full", "CR4"),
                (unix::ETHTOOL_LINK_MODE_100baseT1_Full_BIT, 100, "full", "T1"),
                (unix::ETHTOOL_LINK_MODE_1000baseT1_Full_BIT, 1000, "full", "T1"),
                (unix::ETHTOOL_LINK_MODE_400000baseKR8_Full_BIT, 400000, "full", "KR8"),
                (unix::ETHTOOL_LINK_MODE_400000baseSR8_Full_BIT, 400000, "full", "SR8"),
                (unix::ETHTOOL_LINK_MODE_400000baseLR8_ER8_FR8_Full_BIT, 400000, "full", "LR8_ER8_FR8"),
                (unix::ETHTOOL_LINK_MODE_400000baseDR8_Full_BIT, 400000, "full", "DR8"),
                (unix::ETHTOOL_LINK_MODE_400000baseCR8_Full_BIT, 400000, "full", "CR8"),
                (unix::ETHTOOL_LINK_MODE_100000baseKR_Full_BIT, 100000, "full", "KR"),
                (unix::ETHTOOL_LINK_MODE_100000baseSR_Full_BIT, 100000, "full", "SR"),
                (unix::ETHTOOL_LINK_MODE_100000baseLR_ER_FR_Full_BIT, 100000, "full", "LR_ER_FR"),
                (unix::ETHTOOL_LINK_MODE_100000baseCR_Full_BIT, 100000, "full", "CR"),
                (unix::ETHTOOL_LINK_MODE_100000baseDR_Full_BIT, 100000, "full", "DR"),
                (unix::ETHTOOL_LINK_MODE_200000baseKR2_Full_BIT, 200000, "full", "KR2"),
                (unix::ETHTOOL_LINK_MODE_200000baseSR2_Full_BIT, 200000, "full", "SR2"),
                (unix::ETHTOOL_LINK_MODE_200000baseLR2_ER2_FR2_Full_BIT, 200000, "full", "LR2_ER2_FR2"),
                (unix::ETHTOOL_LINK_MODE_200000baseDR2_Full_BIT, 200000, "full", "DR2"),
                (unix::ETHTOOL_LINK_MODE_200000baseCR2_Full_BIT, 200000, "full", "CR2"),
                (unix::ETHTOOL_LINK_MODE_400000baseKR4_Full_BIT, 400000, "full", "KR4"),
                (unix::ETHTOOL_LINK_MODE_400000baseSR4_Full_BIT, 400000, "full", "SR4"),
                (unix::ETHTOOL_LINK_MODE_400000baseLR4_ER4_FR4_Full_BIT, 400000, "full", "LR4_ER4_FR4"),
                (unix::ETHTOOL_LINK_MODE_400000baseDR4_Full_BIT, 400000, "full", "DR4"),
                (unix::ETHTOOL_LINK_MODE_400000baseCR4_Full_BIT, 400000, "full", "CR4"),
                (unix::ETHTOOL_LINK_MODE_100baseFX_Half_BIT, 100, "half", "FX"),
                (unix::ETHTOOL_LINK_MODE_100baseFX_Full_BIT, 100, "full", "FX"),
            ];
    
            for (bit, speed, duplex, phy) in &speeds {
                if link_modes & (1 << bit) != 0 {
                    ch.collect(Box::new(prometheus::Gauge::new(
                        &link_mode,
                        "Supported speed",
                        &[device, duplex, &format!("{}base{}", speed, phy)],
                        *speed as f64 * MBPS,
                    )));
                }
            }
        }
    }

    fn update(&self, ch: &mut dyn Collector) -> Result<(), Box<dyn std::error::Error>> {
        let net_class = self.fs.net_class()?;
        if net_class.is_empty() {
            return Err("no network devices found".into());
        }

        for device in net_class.keys() {
            if self.device_filter.ignored(device) {
                continue;
            }

            if let Ok(link_info) = self.ethtool.link_info(device) {
                self.update_speeds(ch, "supported", device, link_info.supported);
                self.update_port_info(ch, device, link_info.supported);
                self.update_port_capabilities(ch, "supported", device, link_info.supported);
                self.update_speeds(ch, "advertised", device, link_info.advertising);
                self.update_port_capabilities(ch, "advertised", device, link_info.advertising);
                ch.collect(Box::new(prometheus::Gauge::new(
                    "autonegotiate",
                    "Autonegotiate status",
                    &[device],
                    link_info.autoneg as f64,
                )));
            }

            if let Ok(drv_info) = self.ethtool.driver_info(device) {
                ch.collect(Box::new(prometheus::Gauge::new(
                    "info",
                    "Driver info",
                    &[&drv_info.bus_info, device, &drv_info.driver, &drv_info.erom_version, &drv_info.fw_version, &drv_info.version],
                    1.0,
                )));
            }

            if let Ok(stats) = self.ethtool.stats(device) {
                let metric_fq_names = self.sanitize_and_collect_metrics(&stats, device);
                for (metric_fq_name, metric) in metric_fq_names {
                    ch.collect(Box::new(prometheus::Gauge::new(
                        &metric_fq_name,
                        "Network interface metric",
                        &[device],
                        stats[&metric] as f64,
                    )));
                }
            }
        }

        Ok(())
    }

    fn sanitize_and_collect_metrics(&self, stats: &HashMap<String, u64>, device: &str) -> HashMap<String, String> {
        let mut metric_fq_names = HashMap::new();
        for metric in stats.keys() {
            let metric_name = sanitize_metric_name(metric);
            if !self.metrics_pattern.is_match(&metric_name) {
                continue;
            }
            let metric_fq_name = build_ethtool_fq_name(&metric_name);
            if let Some(existing_metric) = metric_fq_names.insert(metric_fq_name.clone(), metric_name.clone()) {
                self.logger.debug("dropping duplicate metric name", o!(
                    "device" => device,
                    "metricFQName" => metric_fq_name,
                    "metric1" => existing_metric,
                    "metric2" => metric_name
                ));
                metric_fq_names.insert(metric_fq_name, String::new());
            }
        }
        metric_fq_names
    }

    fn entry_with_create(&self, key: &str, metric_fq_name: &str) -> &Desc {
        let mut entries = self.entries.lock().unwrap();
        entries.entry(key.to_string()).or_insert_with(|| {
            Desc::new(
                metric_fq_name.to_string(),
                format!("Network interface {}", key),
                vec!["device".to_string()],
                HashMap::new(),
            )
        })
    }

    fn entry(&self, key: &str) -> &Desc {
        let entries = self.entries.lock().unwrap();
        entries.get(key).unwrap()
    }
}