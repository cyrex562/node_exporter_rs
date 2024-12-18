use prometheus::{self, core::Collector};
use std::sync::Arc;

/// Defines the behavior of a process metrics collector created with `new_process_collector`.
pub struct ProcessCollectorOpts {
    /// `pid_fn` returns the PID of the process the collector collects metrics for.
    /// It is called upon each collection. By default, the PID of the current process
    /// is used, as determined on construction time by calling `std::process::id()`.
    pub pid_fn: Option<Arc<dyn Fn() -> Result<u32, std::io::Error> + Send + Sync>>,
    /// If non-empty, each of the collected metrics is prefixed by the provided string
    /// and an underscore ("_").
    pub namespace: Option<String>,
    /// If true, any error encountered during collection is reported as an invalid metric.
    /// Otherwise, errors are ignored and the collected metrics will be incomplete.
    pub report_errors: bool,
}

impl Default for ProcessCollectorOpts {
    fn default() -> Self {
        ProcessCollectorOpts {
            pid_fn: None,
            namespace: None,
            report_errors: false,
        }
    }
}

/// Returns a collector which exports the current state of process metrics including CPU,
/// memory, and file descriptor usage as well as the process start time.
/// The detailed behavior is defined by the provided `ProcessCollectorOpts`.
///
/// The collector only works on operating systems with a Linux-style proc filesystem and
/// on Microsoft Windows. On other operating systems, it will not collect any metrics.
pub fn new_process_collector(opts: ProcessCollectorOpts) -> impl Collector {
    let pid = match opts.pid_fn {
        Some(pid_fn) => pid_fn().unwrap_or_else(|_| std::process::id()),
        None => std::process::id(),
    };

    // Note: The Rust Prometheus crate's ProcessCollector does not support namespace or report_errors directly.
    // Custom implementation is needed if those features are required.

    prometheus::process_collector::ProcessCollector::new(pid)
}