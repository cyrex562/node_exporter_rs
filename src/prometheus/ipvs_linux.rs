use prometheus::{self, core::{Collector, Desc, Metric, Opts, ValueType}};
use slog::Logger;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

struct IpvsCollector {
    fs: procfs::ProcFs,
    backend_labels: Vec<String>,
    backend_connections_active: TypedDesc,
    backend_connections_inact: TypedDesc,
    backend_weight: TypedDesc,
    connections: TypedDesc,
    incoming_packets: TypedDesc,
    outgoing_packets: TypedDesc,
    incoming_bytes: TypedDesc,
    outgoing_bytes: TypedDesc,
    logger: Logger,
}

struct IpvsBackendStatus {
    active_conn: u64,
    inact_conn: u64,
    weight: u64,
}

const IPVS_LABEL_LOCAL_ADDRESS: &str = "local_address";
const IPVS_LABEL_LOCAL_PORT: &str = "local_port";
const IPVS_LABEL_REMOTE_ADDRESS: &str = "remote_address";
const IPVS_LABEL_REMOTE_PORT: &str = "remote_port";
const IPVS_LABEL_PROTO: &str = "proto";
const IPVS_LABEL_LOCAL_MARK: &str = "local_mark";

lazy_static! {
    static ref FULL_IPVS_BACKEND_LABELS: Vec<&'static str> = vec![
        IPVS_LABEL_LOCAL_ADDRESS,
        IPVS_LABEL_LOCAL_PORT,
        IPVS_LABEL_REMOTE_ADDRESS,
        IPVS_LABEL_REMOTE_PORT,
        IPVS_LABEL_PROTO,
        IPVS_LABEL_LOCAL_MARK,
    ];
    static ref IPVS_LABELS: String = std::env::var("COLLECTOR_IPVS_BACKEND_LABELS").unwrap_or_else(|_| FULL_IPVS_BACKEND_LABELS.join(","));
}

impl IpvsCollector {
    fn new(logger: Logger) -> Result<Self, String> {
        let fs = procfs::ProcFs::new().map_err(|e| format!("failed to open procfs: {}", e))?;
        let backend_labels = Self::parse_ipvs_labels(&IPVS_LABELS)?;

        let connections = TypedDesc {
            desc: Desc::new(
                "node_ipvs_connections_total",
                "The total number of connections made.",
                vec![],
                HashMap::new(),
            ),
            value_type: ValueType::Counter,
        };
        let incoming_packets = TypedDesc {
            desc: Desc::new(
                "node_ipvs_incoming_packets_total",
                "The total number of incoming packets.",
                vec![],
                HashMap::new(),
            ),
            value_type: ValueType::Counter,
        };
        let outgoing_packets = TypedDesc {
            desc: Desc::new(
                "node_ipvs_outgoing_packets_total",
                "The total number of outgoing packets.",
                vec![],
                HashMap::new(),
            ),
            value_type: ValueType::Counter,
        };
        let incoming_bytes = TypedDesc {
            desc: Desc::new(
                "node_ipvs_incoming_bytes_total",
                "The total amount of incoming data.",
                vec![],
                HashMap::new(),
            ),
            value_type: ValueType::Counter,
        };
        let outgoing_bytes = TypedDesc {
            desc: Desc::new(
                "node_ipvs_outgoing_bytes_total",
                "The total amount of outgoing data.",
                vec![],
                HashMap::new(),
            ),
            value_type: ValueType::Counter,
        };
        let backend_connections_active = TypedDesc {
            desc: Desc::new(
                "node_ipvs_backend_connections_active",
                "The current active connections by local and remote address.",
                backend_labels.clone(),
                HashMap::new(),
            ),
            value_type: ValueType::Gauge,
        };
        let backend_connections_inact = TypedDesc {
            desc: Desc::new(
                "node_ipvs_backend_connections_inactive",
                "The current inactive connections by local and remote address.",
                backend_labels.clone(),
                HashMap::new(),
            ),
            value_type: ValueType::Gauge,
        };
        let backend_weight = TypedDesc {
            desc: Desc::new(
                "node_ipvs_backend_weight",
                "The current backend weight by local and remote address.",
                backend_labels.clone(),
                HashMap::new(),
            ),
            value_type: ValueType::Gauge,
        };

        Ok(IpvsCollector {
            fs,
            backend_labels,
            backend_connections_active,
            backend_connections_inact,
            backend_weight,
            connections,
            incoming_packets,
            outgoing_packets,
            incoming_bytes,
            outgoing_bytes,
            logger,
        })
    }

    fn update(&self, ch: &mut dyn FnMut(Box<dyn Metric>)) -> Result<(), String> {
        let ipvs_stats = self.fs.ipvs_stats().map_err(|e| format!("could not get IPVS stats: {}", e))?;
        ch(Box::new(prometheus::Counter::new(self.connections.desc.clone(), ipvs_stats.connections as f64, vec![])));
        ch(Box::new(prometheus::Counter::new(self.incoming_packets.desc.clone(), ipvs_stats.incoming_packets as f64, vec![])));
        ch(Box::new(prometheus::Counter::new(self.outgoing_packets.desc.clone(), ipvs_stats.outgoing_packets as f64, vec![])));
        ch(Box::new(prometheus::Counter::new(self.incoming_bytes.desc.clone(), ipvs_stats.incoming_bytes as f64, vec![])));
        ch(Box::new(prometheus::Counter::new(self.outgoing_bytes.desc.clone(), ipvs_stats.outgoing_bytes as f64, vec![])));

        let backend_stats = self.fs.ipvs_backend_status().map_err(|e| format!("could not get backend status: {}", e))?;

        let mut sums = HashMap::new();
        let mut label_values = HashMap::new();
        for backend in backend_stats {
            let local_address = if backend.local_address.to_string() != "<nil>" {
                backend.local_address.to_string()
            } else {
                String::new()
            };
            let mut kv = vec![String::new(); self.backend_labels.len()];
            for (i, label) in self.backend_labels.iter().enumerate() {
                let label_value = match label.as_str() {
                    IPVS_LABEL_LOCAL_ADDRESS => local_address.clone(),
                    IPVS_LABEL_LOCAL_PORT => backend.local_port.to_string(),
                    IPVS_LABEL_REMOTE_ADDRESS => backend.remote_address.to_string(),
                    IPVS_LABEL_REMOTE_PORT => backend.remote_port.to_string(),
                    IPVS_LABEL_PROTO => backend.proto.clone(),
                    IPVS_LABEL_LOCAL_MARK => backend.local_mark.clone(),
                    _ => String::new(),
                };
                kv[i] = label_value;
            }
            let key = kv.join("-");
            let status = sums.entry(key.clone()).or_insert(IpvsBackendStatus {
                active_conn: 0,
                inact_conn: 0,
                weight: 0,
            });
            status.active_conn += backend.active_conn;
            status.inact_conn += backend.inact_conn;
            status.weight += backend.weight;
            label_values.insert(key, kv);
        }
        for (key, status) in sums {
            let kv = label_values.get(&key).unwrap();
            ch(Box::new(prometheus::Gauge::new(self.backend_connections_active.desc.clone(), status.active_conn as f64, kv.clone())));
            ch(Box::new(prometheus::Gauge::new(self.backend_connections_inact.desc.clone(), status.inact_conn as f64, kv.clone())));
            ch(Box::new(prometheus::Gauge::new(self.backend_weight.desc.clone(), status.weight as f64, kv.clone())));
        }
        Ok(())
    }

    fn parse_ipvs_labels(label_string: &str) -> Result<Vec<String>, String> {
        let labels: Vec<&str> = label_string.split(',').collect();
        let mut label_set = HashMap::new();
        let mut results = Vec::new();
        for label in labels {
            if !label.is_empty() {
                label_set.insert(label, true);
            }
        }

        for label in FULL_IPVS_BACKEND_LABELS.iter() {
            if label_set.contains_key(label) {
                results.push(label.to_string());
            }
            label_set.remove(label);
        }

        if !label_set.is_empty() {
            let keys: Vec<&str> = label_set.keys().cloned().collect();
            return Err(format!("unknown IPVS backend labels: {:?}", keys.join(", ")));
        }

        Ok(results)
    }
}