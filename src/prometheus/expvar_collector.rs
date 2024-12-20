// Systems.
//
// The `exports` map has the following meaning:
//
// The keys in the map correspond to expvar keys, i.e., for every expvar key you
// want to export as a Prometheus metric, you need an entry in the `exports`
// map. The descriptor mapped to each key describes how to export the expvar
// value. It defines the name and the help string of the Prometheus metric
// proxying the expvar value. The type will always be `Untyped`.
//
// For descriptors without variable labels, the expvar value must be a number or
// a bool. The number is then directly exported as the Prometheus sample
// value. (For a bool, `false` translates to 0 and `true` to 1). Expvar values
// that are not numbers or bools are silently ignored.
//
// If the descriptor has one variable label, the expvar value must be an expvar
// map. The keys in the expvar map become the various values of the one
// Prometheus label. The values in the expvar map must be numbers or bools again
// as above.
//
// For descriptors with more than one variable label, the expvar must be a
// nested expvar map, i.e., where the values of the topmost map are maps again
// etc. until a depth is reached that corresponds to the number of labels. The
// leaves of that structure must be numbers or bools as above to serve as the
// sample values.
//
// Anything that does not fit into the scheme above is silently ignored.

use prometheus::{proto::MetricFamily, core::{Collector, Desc, Describer}};
use std::collections::HashMap;
use std::sync::Arc;

pub fn new_expvar_collector(exports: HashMap<String, Desc>) -> impl Collector {
    // Implement function to return a new ExpvarCollector
    ExpvarCollector::new(exports)
}

pub struct ExpvarCollector {
    exports: HashMap<String, Desc>,
}

impl ExpvarCollector {
    pub fn new(exports: HashMap<String, Desc>) -> Self {
        ExpvarCollector { exports }
    }
}

impl Collector for ExpvarCollector {
    fn desc(&self) -> Vec<&Desc> {
        self.exports.values().collect()
    }

    fn collect(&self) -> Vec<MetricFamily> {
        // Implement the collection logic here
        // This would involve reading from expvar and converting values to MetricFamily instances
        Vec::new()
    }
}

use prometheus::{Collector, Desc, Metric, proto::MetricFamily};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

struct ExpvarCollector {
    exports: HashMap<String, Desc>,
}

impl ExpvarCollector {
    fn new(exports: HashMap<String, Desc>) -> Self {
        ExpvarCollector { exports }
    }
}

impl Collector for ExpvarCollector {
    fn describe(&self, descs: &mut Vec<&Desc>) {
        for desc in self.exports.values() {
            descs.push(desc);
        }
    }

    fn collect(&self, mfs: &mut Vec<MetricFamily>) {
        for (name, desc) in &self.exports {
            if let Some(exp_var) = expvar::get(name) {
                let mut labels = vec![String::new(); desc.variable_labels.len()];
                if let Ok(v) = serde_json::from_str::<Value>(&exp_var.to_string()) {
                    self.process_value(&v, 0, &mut labels, desc, mfs);
                } else {
                    mfs.push(prometheus::new_invalid_metric(desc, "Failed to unmarshal expvar"));
                }
            }
        }
    }
}

impl ExpvarCollector {
    fn process_value(&self, v: &Value, i: usize, labels: &mut Vec<String>, desc: &Desc, mfs: &mut Vec<MetricFamily>) {
        if i >= labels.len() {
            let copied_labels = labels.clone();
            match v {
                Value::Number(num) => {
                    if let Some(f) = num.as_f64() {
                        mfs.push(prometheus::new_const_metric(desc, prometheus::proto::MetricType::UNTYPED, f, &copied_labels));
                    }
                }
                Value::Bool(b) => {
                    let val = if *b { 1.0 } else { 0.0 };
                    mfs.push(prometheus::new_const_metric(desc, prometheus::proto::MetricType::UNTYPED, val, &copied_labels));
                }
                _ => {}
            }
            return;
        }

        if let Value::Object(map) = v {
            for (lv, val) in map {
                labels[i] = lv.clone();
                self.process_value(val, i + 1, labels, desc, mfs);
            }
        }
    }
}