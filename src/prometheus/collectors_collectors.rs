use prometheus::{IntGauge, Opts};
use std::collections::HashMap;

pub fn new_build_info_collector() -> IntGauge {
    let mut labels = HashMap::new();
    labels.insert("path".to_string(), env!("CARGO_PKG_NAME").to_string());
    labels.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    labels.insert("checksum".to_string(), "unknown".to_string());

    let build_info_opts = Opts::new("rust_build_info", "Build information")
        .const_labels(labels);

    let build_info = IntGauge::with_opts(build_info_opts).unwrap();
    build_info.set(1);
    build_info
}