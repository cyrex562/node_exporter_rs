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

use prometheus::{core::{Collector, Desc, Metric, Opts}, proto::MetricFamily};
use runtime::metrics::{self, Sample, Description, Kind};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const GO_GC_HEAP_TINY_ALLOCS_OBJECTS: &str = "/gc/heap/tiny/allocs:objects";
const GO_GC_HEAP_ALLOCS_OBJECTS: &str = "/gc/heap/allocs:objects";
const GO_GC_HEAP_FREES_OBJECTS: &str = "/gc/heap/frees:objects";
const GO_GC_HEAP_FREES_BYTES: &str = "/gc/heap/frees:bytes";
const GO_GC_HEAP_ALLOCS_BYTES: &str = "/gc/heap/allocs:bytes";
const GO_GC_HEAP_OBJECTS: &str = "/gc/heap/objects:objects";
const GO_GC_HEAP_GOAL_BYTES: &str = "/gc/heap/goal:bytes";
const GO_MEMORY_CLASSES_TOTAL_BYTES: &str = "/memory/classes/total:bytes";
const GO_MEMORY_CLASSES_HEAP_OBJECTS_BYTES: &str = "/memory/classes/heap/objects:bytes";
const GO_MEMORY_CLASSES_HEAP_UNUSED_BYTES: &str = "/memory/classes/heap/unused:bytes";
const GO_MEMORY_CLASSES_HEAP_RELEASED_BYTES: &str = "/memory/classes/heap/released:bytes";
const GO_MEMORY_CLASSES_HEAP_FREE_BYTES: &str = "/memory/classes/heap/free:bytes";
const GO_MEMORY_CLASSES_HEAP_STACKS_BYTES: &str = "/memory/classes/heap/stacks:bytes";
const GO_MEMORY_CLASSES_OS_STACKS_BYTES: &str = "/memory/classes/os-stacks:bytes";
const GO_MEMORY_CLASSES_METADATA_MSPAN_INUSE_BYTES: &str = "/memory/classes/metadata/mspan/inuse:bytes";
const GO_MEMORY_CLASSES_METADATA_MSPAN_FREE_BYTES: &str = "/memory/classes/metadata/mspan/free:bytes";
const GO_MEMORY_CLASSES_METADATA_MCACHE_INUSE_BYTES: &str = "/memory/classes/metadata/mcache/inuse:bytes";
const GO_MEMORY_CLASSES_METADATA_MCACHE_FREE_BYTES: &str = "/memory/classes/metadata/mcache/free:bytes";
const GO_MEMORY_CLASSES_PROFILING_BUCKETS_BYTES: &str = "/memory/classes/profiling/buckets:bytes";
const GO_MEMORY_CLASSES_METADATA_OTHER_BYTES: &str = "/memory/classes/metadata/other:bytes";
const GO_MEMORY_CLASSES_OTHER_BYTES: &str = "/memory/classes/other:bytes";

struct GoCollector {
    base: BaseGoCollector,
    mu: Mutex<()>,
    sample_buf: Vec<Sample>,
    sample_map: HashMap<String, Sample>,
    rm_exposed_metrics: Vec<CollectorMetric>,
    rm_exact_sum_map_for_hist: HashMap<String, String>,
    ms_metrics: MemStatsMetrics,
    ms_metrics_enabled: bool,
}

impl GoCollector {
    fn new(opts: GoCollectorOptions) -> Self {
        let exposed_descriptions = match_runtime_metrics_rules(&opts.runtime_metric_rules);

        let mut histograms = Vec::new();
        for d in &exposed_descriptions {
            if d.kind == Kind::Float64Histogram {
                histograms.push(Sample::new(d.name.clone()));
            }
        }

        if !histograms.is_empty() {
            metrics::read(&mut histograms);
        }

        let mut buckets_map = HashMap::new();
        for sample in &histograms {
            buckets_map.insert(sample.name.clone(), sample.value.float64_histogram().buckets.clone());
        }

        let mut metric_set = Vec::new();
        let mut sample_buf = Vec::new();
        let mut sample_map = HashMap::new();
        for d in &exposed_descriptions {
            let (namespace, subsystem, name, ok) = runtime_metrics_to_prom(&d.description);
            if !ok {
                continue;
            }
            let help = attach_original_name(&d.description.description, &d.name);

            sample_buf.push(Sample::new(d.name.clone()));
            sample_map.insert(d.name.clone(), sample_buf.last().unwrap().clone());

            let m = if d.kind == Kind::Float64Histogram {
                let has_sum = opts.runtime_metric_sum_for_hist.contains_key(&d.name);
                let unit = &d.name[d.name.find(':').unwrap() + 1..];
                new_batch_histogram(
                    Desc::new(namespace, subsystem, name, help, vec![], HashMap::new()),
                    runtime_metrics_buckets_for_unit(&buckets_map[&d.name], unit),
                    has_sum,
                )
            } else if d.cumulative {
                new_counter(CounterOpts {
                    namespace,
                    subsystem,
                    name,
                    help,
                })
            } else {
                new_gauge(GaugeOpts {
                    namespace,
                    subsystem,
                    name,
                    help,
                })
            };
            metric_set.push(m);
        }

        for h in &histograms {
            if let Some(sum_metric) = opts.runtime_metric_sum_for_hist.get(&h.name) {
                if !sample_map.contains_key(sum_metric) {
                    sample_buf.push(Sample::new(sum_metric.clone()));
                    sample_map.insert(sum_metric.clone(), sample_buf.last().unwrap().clone());
                }
            }
        }

        let ms_metrics = if !opts.disable_mem_stats_like_metrics {
            go_runtime_mem_stats()
        } else {
            MemStatsMetrics::default()
        };

        let ms_descriptions = if !opts.disable_mem_stats_like_metrics {
            best_effort_lookup_rm(&rm_names_for_mem_stats_metrics)
        } else {
            Vec::new()
        };

        for md_desc in &ms_descriptions {
            if !sample_map.contains_key(&md_desc.name) {
                sample_buf.push(Sample::new(md_desc.name.clone()));
                sample_map.insert(md_desc.name.clone(), sample_buf.last().unwrap().clone());
            }
        }

        GoCollector {
            base: new_base_go_collector(),
            sample_buf,
            sample_map,
            rm_exposed_metrics: metric_set,
            rm_exact_sum_map_for_hist: opts.runtime_metric_sum_for_hist,
            ms_metrics,
            ms_metrics_enabled: !opts.disable_mem_stats_like_metrics,
        }
    }

    fn attach_original_name(desc: &str, orig_name: &str) -> String {
        format!("{} Sourced from {}.", desc, orig_name)
    }

    fn unwrap_scalar_rm_value(v: &metrics::Value) -> f64 {
        match v.kind() {
            metrics::Kind::Uint64 => v.uint64() as f64,
            metrics::Kind::Float64 => v.float64(),
            metrics::Kind::Bad => panic!("unexpected bad kind metric"),
            _ => panic!("unexpected unsupported metric: {:?}", v.kind()),
        }
    }

    fn exact_sum_for(&self, rm_name: &str) -> f64 {
        if let Some(sum_name) = self.rm_exact_sum_map_for_hist.get(rm_name) {
            if let Some(s) = self.sample_map.get(sum_name) {
                return self.unwrap_scalar_rm_value(&s.value);
            }
        }
        0.0
    }
}

impl Collector for GoCollector {
    fn desc(&self) -> Vec<&Desc> {
        let mut descs = self.base.desc();
        for i in &self.ms_metrics {
            descs.push(&i.desc);
        }
        for m in &self.rm_exposed_metrics {
            descs.push(m.desc());
        }
        descs
    }

    fn collect(&self) -> Vec<MetricFamily> {
        let mut metrics = self.base.collect();

        if self.sample_buf.is_empty() {
            return metrics;
        }

        let _lock = self.mu.lock().unwrap();

        metrics::read(&mut self.sample_buf);

        for (i, metric) in self.rm_exposed_metrics.iter().enumerate() {
            let sample = &self.sample_buf[i];
            match metric {
                CollectorMetric::Counter(m) => {
                    let v0 = m.get();
                    let v1 = self.unwrap_scalar_rm_value(&sample.value);
                    if v1 > v0 {
                        m.add(v1 - v0);
                    }
                    metrics.push(m.collect());
                }
                CollectorMetric::Gauge(m) => {
                    m.set(self.unwrap_scalar_rm_value(&sample.value));
                    metrics.push(m.collect());
                }
                CollectorMetric::BatchHistogram(m) => {
                    m.update(sample.value.float64_histogram(), self.exact_sum_for(&sample.name));
                    metrics.push(m.collect());
                }
                _ => panic!("unexpected metric type"),
            }
        }

        if self.ms_metrics_enabled {
            let mut ms = runtime::MemStats::default();
            mem_stats_from_rm(&mut ms, &self.sample_map);
            for i in &self.ms_metrics {
                metrics.push(MetricFamily::new(i.desc.clone(), i.val_type, i.eval(&ms)));
            }
        }

        metrics
    }
}

fn mem_stats_from_rm(ms: &mut runtime::MemStats, rm: &HashMap<String, Sample>) {
    let lookup_or_zero = |name: &str| -> u64 {
        rm.get(name).map_or(0, |s| s.value.uint64())
    };

    let tiny_allocs = lookup_or_zero(GO_GC_HEAP_TINY_ALLOCS_OBJECTS);
    ms.mallocs = lookup_or_zero(GO_GC_HEAP_ALLOCS_OBJECTS) + tiny_allocs;
    ms.frees = lookup_or_zero(GO_GC_HEAP_FREES_OBJECTS) + tiny_allocs;

    ms.total_alloc = lookup_or_zero(GO_GC_HEAP_ALLOCS_BYTES);
    ms.sys = lookup_or_zero(GO_MEMORY_CLASSES_TOTAL_BYTES);
    ms.lookups = 0;
    ms.heap_alloc = lookup_or_zero(GO_MEMORY_CLASSES_HEAP_OBJECTS_BYTES);
    ms.alloc = ms.heap_alloc;
    ms.heap_inuse = ms.heap_alloc + lookup_or_zero(GO_MEMORY_CLASSES_HEAP_UNUSED_BYTES);
    ms.heap_released = lookup_or_zero(GO_MEMORY_CLASSES_HEAP_RELEASED_BYTES);
    ms.heap_idle = ms.heap_released + lookup_or_zero(GO_MEMORY_CLASSES_HEAP_FREE_BYTES);
    ms.heap_sys = ms.heap_inuse + ms.heap_idle;
    ms.heap_objects = lookup_or_zero(GO_GC_HEAP_OBJECTS);
    ms.stack_inuse = lookup_or_zero(GO_MEMORY_CLASSES_HEAP_STACKS_BYTES);
    ms.stack_sys = ms.stack_inuse + lookup_or_zero(GO_MEMORY_CLASSES_OS_STACKS_BYTES);
    ms.mspan_inuse = lookup_or_zero(GO_MEMORY_CLASSES_METADATA_MSPAN_INUSE_BYTES);
    ms.mspan_sys = ms.mspan_inuse + lookup_or_zero(GO_MEMORY_CLASSES_METADATA_MSPAN_FREE_BYTES);
    ms.mcache_inuse = lookup_or_zero(GO_MEMORY_CLASSES_METADATA_MCACHE_INUSE_BYTES);
    ms.mcache_sys = ms.mcache_inuse + lookup_or_zero(GO_MEMORY_CLASSES_METADATA_MCACHE_FREE_BYTES);
    ms.buck_hash_sys = lookup_or_zero(GO_MEMORY_CLASSES_PROFILING_BUCKETS_BYTES);
    ms.gcsys = lookup_or_zero(GO_MEMORY_CLASSES_METADATA_OTHER_BYTES);
    ms.other_sys = lookup_or_zero(GO_MEMORY_CLASSES_OTHER_BYTES);
    ms.next_gc = lookup_or_zero(GO_GC_HEAP_GOAL_BYTES);
    ms.gc_cpu_fraction = 0.0;
}

struct BatchHistogram {
    self_collector: SelfCollector,
    desc: Desc,
    has_sum: bool,
    mu: Mutex<()>,
    buckets: Vec<f64>,
    counts: Vec<u64>,
    sum: f64,
}

impl BatchHistogram {
    fn new(desc: Desc, buckets: Vec<f64>, has_sum: bool) -> Self {
        let buckets = if buckets[0] == f64::NEG_INFINITY {
            buckets[1..].to_vec()
        } else {
            buckets
        };
        let counts = vec![0; buckets.len() - 1];
        BatchHistogram {
            self_collector: SelfCollector::new(),
            desc,
            has_sum,
            mu: Mutex::new(()),
            buckets,
            counts,
            sum: 0.0,
        }
    }

    fn update(&self, his: &metrics::Float64Histogram, sum: f64) {
        let counts = &his.counts;
        let buckets = &his.buckets;

        let _lock = self.mu.lock().unwrap();

        for count in &mut self.counts {
            *count = 0;
        }

        let mut j = 0;
        for (i, &count) in counts.iter().enumerate() {
            self.counts[j] += count;
            if buckets[i + 1] == self.buckets[j + 1] {
                j += 1;
            }
        }

        if self.has_sum {
            self.sum = sum;
        }
    }
}

impl Metric for BatchHistogram {
    fn write(&self, metric: &mut MetricFamily) -> Result<(), String> {
        let _lock = self.mu.lock().unwrap();

        let sum = if self.has_sum { self.sum } else { 0.0 };
        let mut dto_buckets = Vec::with_capacity(self.counts.len());
        let mut total_count = 0;
        for (i, &count) in self.counts.iter().enumerate() {
            total_count += count;
            if !self.has_sum && count != 0 {
                sum += self.buckets[i] * count as f64;
            }

            if self.buckets[i + 1].is_infinite() {
                break;
            }

            let upper_bound = f64::from_bits(self.buckets[i + 1].to_bits() - 1);
            dto_buckets.push(prometheus::proto::Bucket {
                cumulative_count: Some(total_count),
                upper_bound: Some(upper_bound),
            });
        }

        metric.histogram = Some(prometheus::proto::Histogram {
            bucket: dto_buckets,
            sample_count: Some(total_count),
            sample_sum: Some(sum),
        });
        Ok(())
    }
}

impl Collector for BatchHistogram {
    fn desc(&self) -> Vec<&Desc> {
        vec![&self.desc]
    }

    fn collect(&self) -> Vec<MetricFamily> {
        let mut metric = MetricFamily::default();
        self.write(&mut metric).unwrap();
        vec![metric]
    }
}

use prometheus::{core::{Collector, Desc, Metric, Opts}, proto::MetricFamily};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn go_runtime_mem_stats() -> Vec<MemStatMetric> {
    vec![
        MemStatMetric::new(
            "alloc_bytes",
            "Number of bytes allocated in heap and currently in use. Equals to /memory/classes/heap/objects:bytes.",
            GaugeValue,
            |ms| ms.alloc as f64,
        ),
        MemStatMetric::new(
            "alloc_bytes_total",
            "Total number of bytes allocated in heap until now, even if released already. Equals to /gc/heap/allocs:bytes.",
            CounterValue,
            |ms| ms.total_alloc as f64,
        ),
        MemStatMetric::new(
            "sys_bytes",
            "Number of bytes obtained from system. Equals to /memory/classes/total:byte.",
            GaugeValue,
            |ms| ms.sys as f64,
        ),
        MemStatMetric::new(
            "mallocs_total",
            "Total number of heap objects allocated, both live and gc-ed. Semantically a counter version for go_memstats_heap_objects gauge. Equals to /gc/heap/allocs:objects + /gc/heap/tiny/allocs:objects.",
            CounterValue,
            |ms| ms.mallocs as f64,
        ),
        MemStatMetric::new(
            "frees_total",
            "Total number of heap objects frees. Equals to /gc/heap/frees:objects + /gc/heap/tiny/allocs:objects.",
            CounterValue,
            |ms| ms.frees as f64,
        ),
        MemStatMetric::new(
            "heap_alloc_bytes",
            "Number of heap bytes allocated and currently in use, same as go_memstats_alloc_bytes. Equals to /memory/classes/heap/objects:bytes.",
            GaugeValue,
            |ms| ms.heap_alloc as f64,
        ),
        MemStatMetric::new(
            "heap_sys_bytes",
            "Number of heap bytes obtained from system. Equals to /memory/classes/heap/objects:bytes + /memory/classes/heap/unused:bytes + /memory/classes/heap/released:bytes + /memory/classes/heap/free:bytes.",
            GaugeValue,
            |ms| ms.heap_sys as f64,
        ),
        MemStatMetric::new(
            "heap_idle_bytes",
            "Number of heap bytes waiting to be used. Equals to /memory/classes/heap/released:bytes + /memory/classes/heap/free:bytes.",
            GaugeValue,
            |ms| ms.heap_idle as f64,
        ),
        MemStatMetric::new(
            "heap_inuse_bytes",
            "Number of heap bytes that are in use. Equals to /memory/classes/heap/objects:bytes + /memory/classes/heap/unused:bytes",
            GaugeValue,
            |ms| ms.heap_inuse as f64,
        ),
        MemStatMetric::new(
            "heap_released_bytes",
            "Number of heap bytes released to OS. Equals to /memory/classes/heap/released:bytes.",
            GaugeValue,
            |ms| ms.heap_released as f64,
        ),
        MemStatMetric::new(
            "heap_objects",
            "Number of currently allocated objects. Equals to /gc/heap/objects:objects.",
            GaugeValue,
            |ms| ms.heap_objects as f64,
        ),
        MemStatMetric::new(
            "stack_inuse_bytes",
            "Number of bytes obtained from system for stack allocator in non-CGO environments. Equals to /memory/classes/heap/stacks:bytes.",
            GaugeValue,
            |ms| ms.stack_inuse as f64,
        ),
        MemStatMetric::new(
            "stack_sys_bytes",
            "Number of bytes obtained from system for stack allocator. Equals to /memory/classes/heap/stacks:bytes + /memory/classes/os-stacks:bytes.",
            GaugeValue,
            |ms| ms.stack_sys as f64,
        ),
        MemStatMetric::new(
            "mspan_inuse_bytes",
            "Number of bytes in use by mspan structures. Equals to /memory/classes/metadata/mspan/inuse:bytes.",
            GaugeValue,
            |ms| ms.mspan_inuse as f64,
        ),
        MemStatMetric::new(
            "mspan_sys_bytes",
            "Number of bytes used for mspan structures obtained from system. Equals to /memory/classes/metadata/mspan/inuse:bytes + /memory/classes/metadata/mspan/free:bytes.",
            GaugeValue,
            |ms| ms.mspan_sys as f64,
        ),
        MemStatMetric::new(
            "mcache_inuse_bytes",
            "Number of bytes in use by mcache structures. Equals to /memory/classes/metadata/mcache/inuse:bytes.",
            GaugeValue,
            |ms| ms.mcache_inuse as f64,
        ),
        MemStatMetric::new(
            "mcache_sys_bytes",
            "Number of bytes used for mcache structures obtained from system. Equals to /memory/classes/metadata/mcache/inuse:bytes + /memory/classes/metadata/mcache/free:bytes.",
            GaugeValue,
            |ms| ms.mcache_sys as f64,
        ),
        MemStatMetric::new(
            "buck_hash_sys_bytes",
            "Number of bytes used by the profiling bucket hash table. Equals to /memory/classes/profiling/buckets:bytes.",
            GaugeValue,
            |ms| ms.buck_hash_sys as f64,
        ),
        MemStatMetric::new(
            "gc_sys_bytes",
            "Number of bytes used for garbage collection system metadata. Equals to /memory/classes/metadata/other:bytes.",
            GaugeValue,
            |ms| ms.gc_sys as f64,
        ),
        MemStatMetric::new(
            "other_sys_bytes",
            "Number of bytes used for other system allocations. Equals to /memory/classes/other:bytes.",
            GaugeValue,
            |ms| ms.other_sys as f64,
        ),
        MemStatMetric::new(
            "next_gc_bytes",
            "Number of heap bytes when next garbage collection will take place. Equals to /gc/heap/goal:bytes.",
            GaugeValue,
            |ms| ms.next_gc as f64,
        ),
    ]
}

struct BaseGoCollector {
    goroutines_desc: Desc,
    threads_desc: Desc,
    gc_desc: Desc,
    gc_last_time_desc: Desc,
    go_info_desc: Desc,
}

impl BaseGoCollector {
    fn new() -> Self {
        BaseGoCollector {
            goroutines_desc: Desc::new(
                "go_goroutines",
                "Number of goroutines that currently exist.",
                vec![],
                HashMap::new(),
            ),
            threads_desc: Desc::new(
                "go_threads",
                "Number of OS threads created.",
                vec![],
                HashMap::new(),
            ),
            gc_desc: Desc::new(
                "go_gc_duration_seconds",
                "A summary of the wall-time pause (stop-the-world) duration in garbage collection cycles.",
                vec![],
                HashMap::new(),
            ),
            gc_last_time_desc: Desc::new(
                "go_memstats_last_gc_time_seconds",
                "Number of seconds since 1970 of last garbage collection.",
                vec![],
                HashMap::new(),
            ),
            go_info_desc: Desc::new(
                "go_info",
                "Information about the Go environment.",
                vec![],
                hashmap!{"version".to_string() => runtime::version()},
            ),
        }
    }
}

impl Collector for BaseGoCollector {
    fn desc(&self) -> Vec<&Desc> {
        vec![
            &self.goroutines_desc,
            &self.threads_desc,
            &self.gc_desc,
            &self.gc_last_time_desc,
            &self.go_info_desc,
        ]
    }

    fn collect(&self) -> Vec<MetricFamily> {
        let mut metrics = Vec::new();

        metrics.push(MetricFamily::new(
            self.goroutines_desc.clone(),
            GaugeValue,
            runtime::num_goroutine() as f64,
        ));

        let n = get_runtime_num_threads();
        metrics.push(MetricFamily::new(
            self.threads_desc.clone(),
            GaugeValue,
            n,
        ));

        let mut stats = debug::GCStats::default();
        stats.pause_quantiles = vec![Duration::new(0, 0); 5];
        debug::read_gc_stats(&mut stats);

        let mut quantiles = HashMap::new();
        for (idx, &pq) in stats.pause_quantiles.iter().enumerate().skip(1) {
            quantiles.insert((idx as f64 + 1.0) / (stats.pause_quantiles.len() as f64 - 1.0), pq.as_secs_f64());
        }
        quantiles.insert(0.0, stats.pause_quantiles[0].as_secs_f64());

        metrics.push(MetricFamily::new_summary(
            self.gc_desc.clone(),
            stats.num_gc as u64,
            stats.pause_total.as_secs_f64(),
            quantiles,
        ));

        metrics.push(MetricFamily::new(
            self.gc_last_time_desc.clone(),
            GaugeValue,
            stats.last_gc.duration_since(UNIX_EPOCH).unwrap().as_secs_f64(),
        ));

        metrics.push(MetricFamily::new(
            self.go_info_desc.clone(),
            GaugeValue,
            1.0,
        ));

        metrics
    }
}

struct MemStatMetric {
    desc: Desc,
    eval: fn(&runtime::MemStats) -> f64,
    val_type: ValueType,
}

impl MemStatMetric {
    fn new(name: &str, help: &str, val_type: ValueType, eval: fn(&runtime::MemStats) -> f64) -> Self {
        MemStatMetric {
            desc: Desc::new(
                &format!("go_memstats_{}", name),
                help,
                vec![],
                HashMap::new(),
            ),
            eval,
            val_type,
        }
    }
}