use prometheus::{core::{Collector, Desc, Metric, Opts}, proto::MetricFamily};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub trait Gauge: Metric + Collector {
    fn set(&self, val: f64);
    fn inc(&self);
    fn dec(&self);
    fn add(&self, val: f64);
    fn sub(&self, val: f64);
    fn set_to_current_time(&self);
}

pub struct GaugeOpts {
    pub namespace: String,
    pub subsystem: String,
    pub name: String,
    pub help: String,
    pub const_labels: HashMap<String, String>,
}

pub struct GaugeVecOpts {
    pub gauge_opts: GaugeOpts,
    pub variable_labels: Vec<String>,
}

pub struct GaugeVec {
    metric_vec: MetricVec,
}

pub struct GaugeImpl {
    val_bits: AtomicU64,
    desc: Desc,
    label_pairs: Vec<LabelPair>,
}

impl Gauge for GaugeImpl {
    fn set(&self, val: f64) {
        self.val_bits.store(val.to_bits(), Ordering::Relaxed);
    }

    fn inc(&self) {
        self.add(1.0);
    }

    fn dec(&self) {
        self.add(-1.0);
    }

    fn add(&self, val: f64) {
        self.update_atomic(val, |old_val, val| old_val + val);
    }

    fn sub(&self, val: f64) {
        self.add(-val);
    }

    fn set_to_current_time(&self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
        self.set(now);
    }
}

impl GaugeImpl {
    fn update_atomic<F>(&self, val: f64, f: F)
    where
        F: Fn(f64, f64) -> f64,
    {
        let mut old_bits = self.val_bits.load(Ordering::Relaxed);
        loop {
            let old_val = f64::from_bits(old_bits);
            let new_val = f(old_val, val);
            let new_bits = new_val.to_bits();
            match self.val_bits.compare_exchange_weak(old_bits, new_bits, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(x) => old_bits = x,
            }
        }
    }
}

impl Metric for GaugeImpl {
    fn write(&self, metric: &mut MetricFamily) -> Result<(), String> {
        let val = f64::from_bits(self.val_bits.load(Ordering::Relaxed));
        populate_metric(GaugeValue, val, &self.label_pairs, None, metric, None)
    }
}

impl Collector for GaugeImpl {
    fn desc(&self) -> Vec<&Desc> {
        vec![&self.desc]
    }

    fn collect(&self) -> Vec<MetricFamily> {
        let mut metric = MetricFamily::default();
        self.write(&mut metric).unwrap();
        vec![metric]
    }
}

impl GaugeVec {
    pub fn new(opts: GaugeOpts, label_names: Vec<String>) -> Self {
        let desc = Desc::new(
            opts.namespace,
            opts.subsystem,
            opts.name,
            opts.help,
            label_names.clone(),
            opts.const_labels.clone(),
        );
        let metric_vec = MetricVec::new(desc, move |label_values| {
            let desc = Desc::new(
                opts.namespace.clone(),
                opts.subsystem.clone(),
                opts.name.clone(),
                opts.help.clone(),
                label_names.clone(),
                opts.const_labels.clone(),
            );
            GaugeImpl {
                val_bits: AtomicU64::new(0),
                desc,
                label_pairs: make_label_pairs(&desc, label_values),
            }
        });
        GaugeVec { metric_vec }
    }

    pub fn get_metric_with_label_values(&self, label_values: &[&str]) -> Result<&GaugeImpl, String> {
        self.metric_vec.get_metric_with_label_values(label_values)
    }

    pub fn get_metric_with(&self, labels: &HashMap<String, String>) -> Result<&GaugeImpl, String> {
        self.metric_vec.get_metric_with(labels)
    }

    pub fn with_label_values(&self, label_values: &[&str]) -> &GaugeImpl {
        self.get_metric_with_label_values(label_values).unwrap()
    }

    pub fn with(&self, labels: &HashMap<String, String>) -> &GaugeImpl {
        self.get_metric_with(labels).unwrap()
    }

    pub fn curry_with(&self, labels: &HashMap<String, String>) -> Result<GaugeVec, String> {
        let metric_vec = self.metric_vec.curry_with(labels)?;
        Ok(GaugeVec { metric_vec })
    }

    pub fn must_curry_with(&self, labels: &HashMap<String, String>) -> GaugeVec {
        self.curry_with(labels).unwrap()
    }
}

pub struct GaugeFunc {
    desc: Desc,
    function: Box<dyn Fn() -> f64 + Send + Sync>,
}

impl GaugeFunc {
    pub fn new(opts: GaugeOpts, function: Box<dyn Fn() -> f64 + Send + Sync>) -> Self {
        let desc = Desc::new(
            opts.namespace,
            opts.subsystem,
            opts.name,
            opts.help,
            vec![],
            opts.const_labels,
        );
        GaugeFunc { desc, function }
    }
}

impl Metric for GaugeFunc {
    fn write(&self, metric: &mut MetricFamily) -> Result<(), String> {
        let val = (self.function)();
        populate_metric(GaugeValue, val, &[], None, metric, None)
    }
}

impl Collector for GaugeFunc {
    fn desc(&self) -> Vec<&Desc> {
        vec![&self.desc]
    }

    fn collect(&self) -> Vec<MetricFamily> {
        let mut metric = MetricFamily::default();
        self.write(&mut metric).unwrap();
        vec![metric]
    }
}