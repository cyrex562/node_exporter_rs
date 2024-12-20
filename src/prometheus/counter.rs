use prometheus::{core::{Collector, Desc, Metric, Opts}, proto::MetricFamily};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

struct Counter {
    desc: Desc,
    val_bits: AtomicU64,
    val_int: AtomicU64,
    created_ts: SystemTime,
}

impl Counter {
    fn new(opts: Opts) -> Self {
        let desc = Desc::new(
            opts.fq_name(),
            opts.help,
            vec![],
            opts.const_labels,
        ).unwrap();
        Self {
            desc,
            val_bits: AtomicU64::new(0),
            val_int: AtomicU64::new(0),
            created_ts: SystemTime::now(),
        }
    }

    fn inc(&self) {
        self.val_int.fetch_add(1, Ordering::SeqCst);
    }

    fn add(&self, v: f64) {
        if v < 0.0 {
            panic!("counter cannot decrease in value");
        }
        let ival = v as u64;
        if ival as f64 == v {
            self.val_int.fetch_add(ival, Ordering::SeqCst);
        } else {
            let mut old_bits = self.val_bits.load(Ordering::SeqCst);
            loop {
                let new_bits = f64::from_bits(old_bits).to_bits() + v.to_bits();
                match self.val_bits.compare_exchange(old_bits, new_bits, Ordering::SeqCst, Ordering::SeqCst) {
                    Ok(_) => break,
                    Err(x) => old_bits = x,
                }
            }
        }
    }

    fn get(&self) -> f64 {
        let fval = f64::from_bits(self.val_bits.load(Ordering::SeqCst));
        let ival = self.val_int.load(Ordering::SeqCst);
        fval + ival as f64
    }
}

impl Metric for Counter {
    fn write(&self, metric: &mut MetricFamily) -> Result<(), Box<dyn std::error::Error>> {
        let val = self.get();
        metric.set_counter(val);
        Ok(())
    }
}

impl Collector for Counter {
    fn desc(&self) -> &Desc {
        &self.desc
    }

    fn collect(&self) -> Vec<MetricFamily> {
        let mut metric = MetricFamily::default();
        self.write(&mut metric).unwrap();
        vec![metric]
    }
}

struct CounterVec {
    desc: Desc,
    counters: Vec<Counter>,
}

impl CounterVec {
    fn new(opts: Opts, label_names: Vec<&str>) -> Self {
        let desc = Desc::new(
            opts.fq_name(),
            opts.help,
            label_names,
            opts.const_labels,
        ).unwrap();
        Self {
            desc,
            counters: Vec::new(),
        }
    }

    fn get_metric_with_label_values(&self, lvs: Vec<&str>) -> Result<&Counter, Box<dyn std::error::Error>> {
        for counter in &self.counters {
            if counter.desc.variable_labels == lvs {
                return Ok(counter);
            }
        }
        Err("Counter not found".into())
    }

    fn with_label_values(&mut self, lvs: Vec<&str>) -> &Counter {
        match self.get_metric_with_label_values(lvs.clone()) {
            Ok(counter) => counter,
            Err(_) => {
                let counter = Counter::new(Opts::new(&self.desc.fq_name, &self.desc.help));
                self.counters.push(counter);
                self.counters.last().unwrap()
            }
        }
    }
}

impl Collector for CounterVec {
    fn desc(&self) -> &Desc {
        &self.desc
    }

    fn collect(&self) -> Vec<MetricFamily> {
        let mut metrics = Vec::new();
        for counter in &self.counters {
            metrics.extend(counter.collect());
        }
        metrics
    }
}