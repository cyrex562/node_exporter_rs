use once_cell::sync::Lazy;
use prometheus::{self, Collector};
use regex::Regex;
use std::sync::Mutex;

pub struct GoRuntimeMetricsRule {
    pub matcher: Regex,
}

impl GoRuntimeMetricsRule {
    pub fn new(pattern: &str) -> Self {
        GoRuntimeMetricsRule {
            matcher: Regex::new(pattern).unwrap(),
        }
    }
}

pub static METRICS_ALL: Lazy<GoRuntimeMetricsRule> =
    Lazy::new(|| GoRuntimeMetricsRule::new("/.*"));
pub static METRICS_GC: Lazy<GoRuntimeMetricsRule> =
    Lazy::new(|| GoRuntimeMetricsRule::new(r"^/gc/.*"));
pub static METRICS_MEMORY: Lazy<GoRuntimeMetricsRule> =
    Lazy::new(|| GoRuntimeMetricsRule::new(r"^/memory/.*"));
pub static METRICS_SCHEDULER: Lazy<GoRuntimeMetricsRule> =
    Lazy::new(|| GoRuntimeMetricsRule::new(r"^/sched/.*"));
pub static METRICS_DEBUG: Lazy<GoRuntimeMetricsRule> =
    Lazy::new(|| GoRuntimeMetricsRule::new(r"^/godebug/.*"));

pub struct GoCollectorOptions {
    pub disable_mem_stats_like_metrics: bool,
    pub runtime_metric_rules: Vec<GoCollectorRule>,
}

impl Default for GoCollectorOptions {
    fn default() -> Self {
        GoCollectorOptions {
            disable_mem_stats_like_metrics: false,
            runtime_metric_rules: Vec::new(),
        }
    }
}

pub struct GoCollectorRule {
    pub matcher: Regex,
    pub deny: bool,
}

pub fn with_go_collector_mem_stats_metrics_disabled(
) -> impl FnOnce(&mut GoCollectorOptions) {
    |options: &mut GoCollectorOptions| {
        options.disable_mem_stats_like_metrics = true;
    }
}

pub fn with_go_collector_runtime_metrics(
    rules: Vec<GoRuntimeMetricsRule>,
) -> impl FnOnce(&mut GoCollectorOptions) {
    move |options: &mut GoCollectorOptions| {
        let rs = rules
            .into_iter()
            .map(|r| GoCollectorRule {
                matcher: r.matcher,
                deny: false,
            })
            .collect::<Vec<_>>();
        options.runtime_metric_rules.extend(rs);
    }
}

pub fn without_go_collector_runtime_metrics(
    matchers: Vec<Regex>,
) -> impl FnOnce(&mut GoCollectorOptions) {
    move |options: &mut GoCollectorOptions| {
        let rs = matchers
            .into_iter()
            .map(|m| GoCollectorRule {
                matcher: m,
                deny: true,
            })
            .collect::<Vec<_>>();
        options.runtime_metric_rules.extend(rs);
    }
}

pub fn new_go_collector(
    opts: Vec<impl FnOnce(&mut GoCollectorOptions)>,
) -> impl Collector {
    let mut options = GoCollectorOptions::default();
    for opt in opts {
        opt(&mut options);
    }
    GoCollector::new(options)
}

pub struct GoCollector {
    options: GoCollectorOptions,
}

impl GoCollector {
    pub fn new(options: GoCollectorOptions) -> Self {
        GoCollector { options }
    }
}

impl Collector for GoCollector {
    fn desc(&self) -> Vec<&prometheus::core::Desc> {
        Vec::new()
    }

    fn collect(&self) -> Vec<prometheus::proto::MetricFamily> {
        Vec::new()
    }
}