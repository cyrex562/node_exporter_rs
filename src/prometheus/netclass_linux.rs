use regex::Regex;
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug)]
pub struct NetClassCollector {
    fs: SysFs,
    subsystem: String,
    ignored_devices_pattern: Regex,
    metric_descs: Mutex<HashMap<String, prometheus::Desc>>,
    logger: slog::Logger,
}

#[derive(Debug, Error)]
pub enum NetClassCollectorError {
    #[error("failed to open sysfs")]
    SysFsError(#[from] sysfs::Error),
    #[error("could not get net class info")]
    NetClassInfoError(#[from] sysfs::Error),
}

impl NetClassCollector {
    pub fn new(logger: slog::Logger) -> Result<Self, NetClassCollectorError> {
        let fs = SysFs::new()?;
        let pattern = Regex::new("^$").unwrap();
        Ok(Self {
            fs,
            subsystem: "network".to_string(),
            ignored_devices_pattern: pattern,
            metric_descs: Mutex::new(HashMap::new()),
            logger,
        })
    }

    pub fn update(&self, ch: &mut dyn FnMut(prometheus::Metric)) -> Result<(), NetClassCollectorError> {
        self.net_class_sysfs_update(ch)
    }

    fn net_class_sysfs_update(&self, ch: &mut dyn FnMut(prometheus::Metric)) -> Result<(), NetClassCollectorError> {
        let net_class = self.get_net_class_info()?;
        for iface_info in net_class.values() {
            let up_desc = prometheus::Desc::new(
                prometheus::Opts::new(
                    "up",
                    "Value is 1 if operstate is 'up', 0 otherwise.",
                )
                .namespace("namespace")
                .subsystem(&self.subsystem)
                .variable_labels(vec!["device".to_string()]),
            );

            let up_value = if iface_info.oper_state == "up" { 1.0 } else { 0.0 };
            ch(prometheus::Gauge::new(up_desc.clone(), up_value, vec![iface_info.name.clone()]));

            let info_desc = prometheus::Desc::new(
                prometheus::Opts::new(
                    "info",
                    "Non-numeric data from /sys/class/net/<iface>, value is always 1.",
                )
                .namespace("namespace")
                .subsystem(&self.subsystem)
                .variable_labels(vec![
                    "device".to_string(),
                    "address".to_string(),
                    "broadcast".to_string(),
                    "duplex".to_string(),
                    "operstate".to_string(),
                    "adminstate".to_string(),
                    "ifalias".to_string(),
                ]),
            );

            ch(prometheus::Gauge::new(
                info_desc.clone(),
                1.0,
                vec![
                    iface_info.name.clone(),
                    iface_info.address.clone(),
                    iface_info.broadcast.clone(),
                    iface_info.duplex.clone(),
                    iface_info.oper_state.clone(),
                    get_admin_state(iface_info.flags),
                    iface_info.if_alias.clone(),
                ],
            ));

            self.push_metric(ch, "address_assign_type", iface_info.addr_assign_type, iface_info.name.clone());
            self.push_metric(ch, "carrier", iface_info.carrier, iface_info.name.clone());
            self.push_metric(ch, "carrier_changes_total", iface_info.carrier_changes, iface_info.name.clone());
            self.push_metric(ch, "carrier_up_changes_total", iface_info.carrier_up_count, iface_info.name.clone());
            self.push_metric(ch, "carrier_down_changes_total", iface_info.carrier_down_count, iface_info.name.clone());
            self.push_metric(ch, "device_id", iface_info.dev_id, iface_info.name.clone());
            self.push_metric(ch, "dormant", iface_info.dormant, iface_info.name.clone());
            self.push_metric(ch, "flags", iface_info.flags, iface_info.name.clone());
            self.push_metric(ch, "iface_id", iface_info.if_index, iface_info.name.clone());
            self.push_metric(ch, "iface_link", iface_info.if_link, iface_info.name.clone());
            self.push_metric(ch, "iface_link_mode", iface_info.link_mode, iface_info.name.clone());
            self.push_metric(ch, "mtu_bytes", iface_info.mtu, iface_info.name.clone());
            self.push_metric(ch, "name_assign_type", iface_info.name_assign_type, iface_info.name.clone());
            self.push_metric(ch, "net_dev_group", iface_info.net_dev_group, iface_info.name.clone());

            if let Some(speed) = iface_info.speed {
                if speed >= 0 {
                    let speed_bytes = (speed * 1000 * 1000 / 8) as i64;
                    self.push_metric(ch, "speed_bytes", speed_bytes,