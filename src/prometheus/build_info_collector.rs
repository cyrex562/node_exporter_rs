use prometheus::{self, core::{Collector, Desc, Opts}, proto::MetricFamily};
use std::sync::Arc;
use std::collections::HashMap;

struct BuildInfoCollector {
    desc: Arc<Desc>,
    labels: HashMap<String, String>,
}

impl BuildInfoCollector {
    fn new() -> Self {
        let (path, version, sum) = match std::env::var("CARGO_PKG_NAME") {
            Ok(path) => (
                path,
                std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string()),
                std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string()),
            ),
            Err(_) => ("unknown".to_string(), "unknown".to_string(), "unknown".to_string()),
        };

        let desc = Desc::new(
            "rust_build_info".to_string(),
            "Build information about the main Rust crate.".to_string(),
            vec![],
            HashMap::from([
                ("path".to_string(), path),
                ("version".to_string(), version),
                ("checksum".to_string(), sum),
            ]),
        );

        Self {
            desc: Arc::new(desc),
            labels: HashMap::new(),
        }
    }
}

impl Collector for BuildInfoCollector {
    fn collect(&self) -> Vec<MetricFamily> {
        let metric = prometheus::core::MetricFamily::new(
            self.desc.clone(),
            prometheus::proto::MetricType::GAUGE,
            1.0,
            vec![],
        );
        vec![metric]
    }
}