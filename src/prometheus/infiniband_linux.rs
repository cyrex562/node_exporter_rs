use prometheus::{self, core::{Collector, Desc, Metric, Opts}};
use slog::Logger;
use std::collections::HashMap;
use std::sync::Arc;
use sysfs::SysFs;

struct InfiniBandCollector {
    fs: SysFs,
    metric_descs: HashMap<String, Desc>,
    logger: Logger,
    subsystem: String,
}

impl InfiniBandCollector {
    fn new(logger: Logger) -> Result<Self, String> {
        let fs = SysFs::new().map_err(|e| format!("failed to open sysfs: {}", e))?;
        let subsystem = "infiniband".to_string();

        let descriptions = vec![
            ("legacy_multicast_packets_received_total", "Number of multicast packets received"),
            ("legacy_multicast_packets_transmitted_total", "Number of multicast packets transmitted"),
            ("legacy_data_received_bytes_total", "Number of data octets received on all links"),
            ("legacy_packets_received_total", "Number of data packets received on all links"),
            ("legacy_unicast_packets_received_total", "Number of unicast packets received"),
            ("legacy_unicast_packets_transmitted_total", "Number of unicast packets transmitted"),
            ("legacy_data_transmitted_bytes_total", "Number of data octets transmitted on all links"),
            ("legacy_packets_transmitted_total", "Number of data packets received on all links"),
            ("excessive_buffer_overrun_errors_total", "Number of times that OverrunErrors consecutive flow control update periods occurred, each having at least one overrun error."),
            ("link_downed_total", "Number of times the link failed to recover from an error state and went down"),
            ("link_error_recovery_total", "Number of times the link successfully recovered from an error state"),
            ("local_link_integrity_errors_total", "Number of times that the count of local physical errors exceeded the threshold specified by LocalPhyErrors."),
            ("multicast_packets_received_total", "Number of multicast packets received (including errors)"),
            ("multicast_packets_transmitted_total", "Number of multicast packets transmitted (including errors)"),
            ("physical_state_id", "Physical state of the InfiniBand port (0: no change, 1: sleep, 2: polling, 3: disable, 4: shift, 5: link up, 6: link error recover, 7: phytest)"),
            ("port_constraint_errors_received_total", "Number of packets received on the switch physical port that are discarded"),
            ("port_constraint_errors_transmitted_total", "Number of packets not transmitted from the switch physical port"),
            ("port_data_received_bytes_total", "Number of data octets received on all links"),
            ("port_data_transmitted_bytes_total", "Number of data octets transmitted on all links"),
            ("port_discards_received_total", "Number of inbound packets discarded by the port because the port is down or congested"),
            ("port_discards_transmitted_total", "Number of outbound packets discarded by the port because the port is down or congested"),
            ("port_errors_received_total", "Number of packets containing an error that were received on this port"),
            ("port_packets_received_total", "Number of packets received on all VLs by this port (including errors)"),
            ("port_packets_transmitted_total", "Number of packets transmitted on all VLs from this port (including errors)"),
            ("port_transmit_wait_total", "Number of ticks during which the port had data to transmit but no data was sent during the entire tick"),
            ("rate_bytes_per_second", "Maximum signal transfer rate"),
            ("state_id", "State of the InfiniBand port (0: no change, 1: down, 2: init, 3: armed, 4: active, 5: act defer)"),
            ("unicast_packets_received_total", "Number of unicast packets received (including errors)"),
            ("unicast_packets_transmitted_total", "Number of unicast packets transmitted (including errors)"),
            ("port_receive_remote_physical_errors_total", "Number of packets marked with the EBP (End of Bad Packet) delimiter received on the port."),
            ("port_receive_switch_relay_errors_total", "Number of packets that could not be forwarded by the switch."),
            ("symbol_error_total", "Number of minor link errors detected on one or more physical lanes."),
            ("vl15_dropped_total", "Number of incoming VL15 packets dropped due to resource limitations."),
        ];

        let metric_descs = descriptions.into_iter().map(|(name, desc)| {
            let desc = prometheus::Desc::new(
                prometheus::core::Opts::new(name, desc)
                    .namespace("node")
                    .subsystem(&subsystem)
                    .const_labels(HashMap::new()),
                vec!["device".to_string(), "port".to_string()],
            );
            (name.to_string(), desc)
        }).collect();

        Ok(InfiniBandCollector {
            fs,
            metric_descs,
            logger,
            subsystem,
        })
    }

    fn push_metric(&self, ch: &mut dyn FnMut(Box<dyn Metric>), name: &str, value: u64, device_name: &str, port: &str, value_type: prometheus::core::ValueType) {
        if let Some(desc) = self.metric_descs.get(name) {
            ch(Box::new(prometheus::Gauge::new(desc.clone(), value as f64, vec![device_name.to_string(), port.to_string()])));
        }
    }

    fn push_counter(&self, ch: &mut dyn FnMut(Box<dyn Metric>), name: &str, value: Option<u64>, device_name: &str, port: &str) {
        if let Some(value) = value {
            self.push_metric(ch, name, value, device_name, port, prometheus::core::ValueType::Counter);
        }
    }

    fn update(&self, ch: &mut dyn FnMut(Box<dyn Metric>)) -> Result<(), String> {
        let devices = self.fs.infiniband_class().map_err(|e| format!("error obtaining InfiniBand class info: {}", e))?;

        for device in devices {
            let info_desc = prometheus::Desc::new(
                prometheus::core::Opts::new("info", "Non-numeric data from /sys/class/infiniband/<device>, value is always 1.")
                    .namespace("node")
                    .subsystem(&self.subsystem)
                    .const_labels(HashMap::new()),
                vec!["device".to_string(), "board_id".to_string(), "firmware_version".to_string(), "hca_type".to_string()],
            );
            ch(Box::new(prometheus::Gauge::new(info_desc, 1.0, vec![device.name.clone(), device.board_id.clone(), device.firmware_version.clone(), device.hca_type.clone()])));

            for port in device.ports {
                let port_str = port.port.to_string();

                self.push_metric(ch, "state_id", port.state_id as u64, &device.name, &port_str, prometheus::core::ValueType::Gauge);
                self.push_metric(ch, "physical_state_id", port.phys_state_id as u64, &device.name, &port_str, prometheus::core::ValueType::Gauge);
                self.push_metric(ch, "rate_bytes_per_second", port.rate, &device.name, &port_str, prometheus::core::ValueType::Gauge);

                self.push_counter(ch, "legacy_multicast_packets_received_total", port.counters.legacy_port_multicast_rcv_packets, &device.name, &port_str);
                self.push_counter(ch, "legacy_multicast_packets_transmitted_total", port.counters.legacy_port_multicast_xmit_packets, &device.name, &port_str);
                self.push_counter(ch, "legacy_data_received_bytes_total", port.counters.legacy_port_rcv_data64, &device.name, &port_str);
                self.push_counter(ch, "legacy_packets_received_total", port.counters.legacy_port_rcv_packets64, &device.name, &port_str);
                self.push_counter(ch, "legacy_unicast_packets_received_total", port.counters.legacy_port_unicast_rcv_packets, &device.name, &port_str);
                self.push_counter(ch, "legacy_unicast_packets_transmitted_total", port.counters.legacy_port_unicast_xmit_packets, &device.name, &port_str);
                self.push_counter(ch, "legacy_data_transmitted_bytes_total", port.counters.legacy_port_xmit_data64, &device.name, &port_str);
                self.push_counter(ch, "legacy_packets_transmitted_total", port.counters.legacy_port_xmit_packets64, &device.name, &port_str);
                self.push_counter(ch, "excessive_buffer_overrun_errors_total", port.counters.excessive_buffer_overrun_errors, &device.name, &port_str);
                self.push_counter(ch, "link_downed_total", port.counters.link_downed, &device.name, &port_str);
                self.push_counter(ch, "link_error_recovery_total", port.counters.link_error_recovery, &device.name, &port_str);
                self.push_counter(ch, "local_link_integrity_errors_total", port.counters.local_link_integrity_errors, &device.name, &port_str);
                self.push_counter(ch, "multicast_packets_received_total", port.counters.multicast_rcv_packets, &device.name, &port_str);
                self.push_counter(ch, "multicast_packets_transmitted_total", port.counters.multicast_xmit_packets, &device.name, &port_str);
                self.push_counter(ch, "port_constraint_errors_received_total", port.counters.port_rcv_constraint_errors, &device.name, &port_str);
                self.push_counter(ch, "port_constraint_errors_transmitted_total", port.counters.port_xmit_constraint_errors, &device.name, &port_str);
                self.push_counter(ch, "port_data_received_bytes_total", port.counters.port_rcv_data, &device.name, &port_str);
                self.push_counter(ch, "port_data_transmitted_bytes_total", port.counters.port_xmit_data, &device.name, &port_str);
                self.push_counter(ch, "port_discards_received_total", port.counters.port_rcv_discards, &device.name, &port_str);
                self.push_counter(ch, "port_discards_transmitted_total", port.counters.port_xmit_discards, &device.name, &port_str);
                self.push_counter(ch, "port_errors_received_total", port.counters.port_rcv_errors, &device.name, &port_str);
                self.push_counter(ch, "port_packets_received_total", port.counters.port_rcv_packets, &device.name, &port_str);
                self.push_counter(ch, "port_packets_transmitted_total", port.counters.port_xmit_packets, &device.name, &port_str);
                self.push_counter(ch, "port_transmit_wait_total", port.counters.port_xmit_wait, &device.name, &port_str);
                self.push_counter(ch, "unicast_packets_received_total", port.counters.unicast_rcv_packets, &device.name, &port_str);
                self.push_counter(ch, "unicast_packets_transmitted_total", port.counters.unicast_xmit_packets, &device.name, &port_str);
                self.push_counter(ch, "port_receive_remote_physical_errors_total", port.counters.port_rcv_remote_physical_errors, &device.name, &port_str);
                self.push_counter(ch, "port_receive_switch_relay_errors_total", port.counters.port_rcv_switch_relay_errors, &device.name, &port_str);
                self.push_counter(ch, "symbol_error_total", port.counters.symbol_error, &device.name, &port_str);
                self.push_counter(ch, "vl15_dropped_total", port.counters.vl15_dropped, &device.name, &port_str);
            }
        }

        Ok(())
    }
}