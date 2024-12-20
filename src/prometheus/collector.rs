use prometheus::{core::{Collector, Desc, Metric}, proto::MetricFamily};
use std::sync::mpsc::Sender;

trait Collector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc));
    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>));
}

fn describe_by_collect<C: Collector>(collector: C, descs: &mut dyn FnMut(&Desc)) {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        collector.collect(&mut |metric| {
            tx.send(metric.desc()).unwrap();
        });
    });
    for desc in rx {
        descs(desc);
    }
}

struct SelfCollector {
    self_metric: Box<dyn Metric>,
}

impl SelfCollector {
    fn init(&mut self, self_metric: Box<dyn Metric>) {
        self.self_metric = self_metric;
    }
}

impl Collector for SelfCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(self.self_metric.desc());
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        metrics(self.self_metric.clone());
    }
}

trait CollectorMetric: Metric + Collector {}

use prometheus::{self, core::{Collector, Desc, Metric, Opts}, proto::MetricFamily};
use slog::Logger;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const NAMESPACE: &str = "node";

lazy_static! {
    static ref SCRAPE_DURATION_DESC: Desc = Desc::new(
        prometheus::core::build_fq_name(NAMESPACE, "scrape", "collector_duration_seconds"),
        "node_exporter: Duration of a collector scrape.",
        vec!["collector".to_string()],
        HashMap::new()
    ).unwrap();
    static ref SCRAPE_SUCCESS_DESC: Desc = Desc::new(
        prometheus::core::build_fq_name(NAMESPACE, "scrape", "collector_success"),
        "node_exporter: Whether a collector succeeded.",
        vec!["collector".to_string()],
        HashMap::new()
    ).unwrap();
}

lazy_static! {
    static ref FACTORIES: Mutex<HashMap<String, Box<dyn Fn(&Logger) -> Result<Box<dyn Collector>, Box<dyn std::error::Error>>>>> = Mutex::new(HashMap::new());
    static ref INITIATED_COLLECTORS: Mutex<HashMap<String, Box<dyn Collector>>> = Mutex::new(HashMap::new());
    static ref COLLECTOR_STATE: Mutex<HashMap<String, bool>> = Mutex::new(HashMap::new());
    static ref FORCED_COLLECTORS: Mutex<HashMap<String, bool>> = Mutex::new(HashMap::new());
}

struct NodeCollector {
    collectors: HashMap<String, Box<dyn Collector>>,
    logger: Logger,
}

impl NodeCollector {
    fn disable_default_collectors() {
        let mut state = COLLECTOR_STATE.lock().unwrap();
        let forced = FORCED_COLLECTORS.lock().unwrap();
        for (key, value) in state.iter_mut() {
            if !forced.contains_key(key) {
                *value = false;
            }
        }
    }

    fn new(logger: Logger, filters: Vec<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut collectors = HashMap::new();
        let state = COLLECTOR_STATE.lock().unwrap();
        let mut initiated = INITIATED_COLLECTORS.lock().unwrap();
        for filter in filters {
            if let Some(enabled) = state.get(&filter) {
                if !enabled {
                    return Err(format!("disabled collector: {}", filter).into());
                }
            } else {
                return Err(format!("missing collector: {}", filter).into());
            }
        }
        for (key, enabled) in state.iter() {
            if *enabled {
                if let Some(collector) = initiated.get(key) {
                    collectors.insert(key.clone(), collector.clone());
                } else {
                    let factory = FACTORIES.lock().unwrap().get(key).unwrap();
                    let collector = factory(&logger.new(o!("collector" => key.clone())))?;
                    collectors.insert(key.clone(), collector.clone());
                    initiated.insert(key.clone(), collector);
                }
            }
        }
        Ok(Self { collectors, logger })
    }
}

impl Collector for NodeCollector {
    fn describe(&self, descs: &mut dyn FnMut(&Desc)) {
        descs(&SCRAPE_DURATION_DESC);
        descs(&SCRAPE_SUCCESS_DESC);
    }

    fn collect(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) {
        let mut handles = vec![];
        for (name, collector) in &self.collectors {
            let logger = self.logger.clone();
            let name = name.clone();
            let collector = collector.clone();
            let metrics = metrics.clone();
            handles.push(std::thread::spawn(move || {
                execute(&name, &*collector, &metrics, &logger);
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }
    }
}

fn execute(name: &str, collector: &dyn Collector, metrics: &mut dyn FnMut(Box<dyn Metric>), logger: &Logger) {
    let start = std::time::Instant::now();
    let result = collector.update(metrics);
    let duration = start.elapsed();
    let success = if result.is_ok() { 1.0 } else { 0.0 };
    if let Err(err) = result {
        if is_no_data_error(&err) {
            logger.debug("collector returned no data", o!("name" => name, "duration_seconds" => duration.as_secs_f64(), "err" => err.to_string()));
        } else {
            logger.error("collector failed", o!("name" => name, "duration_seconds" => duration.as_secs_f64(), "err" => err.to_string()));
        }
    } else {
        logger.debug("collector succeeded", o!("name" => name, "duration_seconds" => duration.as_secs_f64()));
    }
    metrics(Box::new(prometheus::core::MetricFamily::new(
        SCRAPE_DURATION_DESC.clone(),
        prometheus::proto::MetricType::GAUGE,
        duration.as_secs_f64(),
        vec![name.to_string()],
    )));
    metrics(Box::new(prometheus::core::MetricFamily::new(
        SCRAPE_SUCCESS_DESC.clone(),
        prometheus::proto::MetricType::GAUGE,
        success,
        vec![name.to_string()],
    )));
}

trait Collector {
    fn update(&self, metrics: &mut dyn FnMut(Box<dyn Metric>)) -> Result<(), Box<dyn std::error::Error>>;
}

struct TypedDesc {
    desc: Desc,
    value_type: prometheus::proto::MetricType,
}

impl TypedDesc {
    fn must_new_const_metric(&self, value: f64, labels: Vec<String>) -> Box<dyn Metric> {
        Box::new(prometheus::core::MetricFamily::new(
            self.desc.clone(),
            self.value_type,
            value,
            labels,
        ))
    }
}

#[derive(Debug)]
struct NoDataError;

impl std::fmt::Display for NoDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "collector returned no data")
    }
}

impl std::error::Error for NoDataError {}

fn is_no_data_error(err: &Box<dyn std::error::Error>) -> bool {
    err.downcast_ref::<NoDataError>().is_some()
}

fn push_metric(ch: &mut dyn FnMut(Box<dyn Metric>), field_desc: &Desc, name: &str, value: impl Into<f64>, value_type: prometheus::proto::MetricType, label_values: Vec<String>) {
    let f_val = value.into();
    ch(Box::new(prometheus::core::MetricFamily::new(
        field_desc.clone(),
        value_type,
        f_val,
        label_values,
    )));
}