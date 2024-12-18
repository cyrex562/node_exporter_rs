use prometheus::{opts, Encoder, Gauge, Registry, TextEncoder};
use std::collections::HashMap;

pub fn new_collector(program: &str) -> Gauge {
    let gauge_opts = opts!(
        "build_info",
        format!(
            "A metric with a constant '1' value labeled by version, revision, branch, goversion from which {} was built, and the goos and goarch for the build.",
            program
        )
    )
    .namespace(program)
    .const_labels(get_labels());

    let gauge = Gauge::with_opts(gauge_opts).unwrap();
    gauge.set(1.0);
    gauge
}

fn get_labels() -> HashMap<String, String> {
    let mut labels = HashMap::new();
    labels.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    labels.insert("revision".to_string(), get_revision());
    labels.insert("branch".to_string(), get_branch());
    labels.insert("goversion".to_string(), get_rustc_version());
    labels.insert("goos".to_string(), std::env::consts::OS.to_string());
    labels.insert("goarch".to_string(), std::env::consts::ARCH.to_string());
    labels.insert("tags".to_string(), get_tags());
    labels
}

fn get_revision() -> String {
    // Implement function to get the git revision
    "unknown".to_string()
}

fn get_branch() -> String {
    // Implement function to get the git branch
    "unknown".to_string()
}

fn get_rustc_version() -> String {
    let version_meta = rustc_version_runtime::version_meta();
    version_meta.semver.to_string()
}

fn get_tags() -> String {
    // Implement function to get any tags
    "unknown".to_string()
}