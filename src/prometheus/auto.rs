// Copyright 2018 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Package promauto provides alternative constructors for the fundamental
//! Prometheus metric types and their …Vec and …Func variants. The difference to
//! their counterparts in the prometheus package is that the promauto
//! constructors register the Collectors with a registry before returning them.
//! There are two sets of constructors. The constructors in the first set are
//! top-level functions, while the constructors in the other set are methods of
//! the Factory type. The top-level functions return Collectors registered with
//! the global registry (prometheus::default_registry()), while the methods return
//! Collectors registered with the registry the Factory was constructed with. All
//! constructors panic if the registration fails.

use prometheus::{self, Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec, Summary, SummaryVec, Untyped, Registry, Opts, CounterOpts, GaugeOpts, HistogramOpts, SummaryOpts, UntypedOpts};

pub fn new_counter(opts: CounterOpts) -> Counter {
    with(prometheus::default_registry()).new_counter(opts)
}

pub fn new_counter_vec(opts: CounterOpts, label_names: &[&str]) -> CounterVec {
    with(prometheus::default_registry()).new_counter_vec(opts, label_names)
}

pub fn new_counter_func<F>(opts: CounterOpts, function: F) -> Counter
where
    F: Fn() -> f64 + 'static,
{
    with(prometheus::default_registry()).new_counter_func(opts, function)
}

pub fn new_gauge(opts: GaugeOpts) -> Gauge {
    with(prometheus::default_registry()).new_gauge(opts)
}

pub fn new_gauge_vec(opts: GaugeOpts, label_names: &[&str]) -> GaugeVec {
    with(prometheus::default_registry()).new_gauge_vec(opts, label_names)
}

pub fn new_gauge_func<F>(opts: GaugeOpts, function: F) -> Gauge
where
    F: Fn() -> f64 + 'static,
{
    with(prometheus::default_registry()).new_gauge_func(opts, function)
}

pub fn new_summary(opts: SummaryOpts) -> Summary {
    with(prometheus::default_registry()).new_summary(opts)
}

pub fn new_summary_vec(opts: SummaryOpts, label_names: &[&str]) -> SummaryVec {
    with(prometheus::default_registry()).new_summary_vec(opts, label_names)
}

pub fn new_histogram(opts: HistogramOpts) -> Histogram {
    with(prometheus::default_registry()).new_histogram(opts)
}

pub fn new_histogram_vec(opts: HistogramOpts, label_names: &[&str]) -> HistogramVec {
    with(prometheus::default_registry()).new_histogram_vec(opts, label_names)
}

pub fn new_untyped_func<F>(opts: UntypedOpts, function: F) -> Untyped
where
    F: Fn() -> f64 + 'static,
{
    with(prometheus::default_registry()).new_untyped_func(opts, function)
}

pub struct Factory {
    registry: Option<Registry>,
}

pub fn with(registry: Registry) -> Factory {
    Factory { registry: Some(registry) }
}

impl Factory {
    pub fn new_counter(&self, opts: CounterOpts) -> Counter {
        let counter = Counter::with_opts(opts).unwrap();
        if let Some(ref registry) = self.registry {
            registry.register(Box::new(counter.clone())).unwrap();
        }
        counter
    }

    pub fn new_counter_vec(&self, opts: CounterOpts, label_names: &[&str]) -> CounterVec {
        let counter_vec = CounterVec::new(opts, label_names).unwrap();
        if let Some(ref registry) = self.registry {
            registry.register(Box::new(counter_vec.clone())).unwrap();
        }
        counter_vec
    }

    pub fn new_counter_func<F>(&self, opts: CounterOpts, function: F) -> Counter
    where
        F: Fn() -> f64 + 'static,
    {
        let counter_func = Counter::with_opts_and_function(opts, function).unwrap();
        if let Some(ref registry) = self.registry {
            registry.register(Box::new(counter_func.clone())).unwrap();
        }
        counter_func
    }

    pub fn new_gauge(&self, opts: GaugeOpts) -> Gauge {
        let gauge = Gauge::with_opts(opts).unwrap();
        if let Some(ref registry) = self.registry {
            registry.register(Box::new(gauge.clone())).unwrap();
        }
        gauge
    }

    pub fn new_gauge_vec(&self, opts: GaugeOpts, label_names: &[&str]) -> GaugeVec {
        let gauge_vec = GaugeVec::new(opts, label_names).unwrap();
        if let Some(ref registry) = self.registry {
            registry.register(Box::new(gauge_vec.clone())).unwrap();
        }
        gauge_vec
    }

    pub fn new_gauge_func<F>(&self, opts: GaugeOpts, function: F) -> Gauge
    where
        F: Fn() -> f64 + 'static,
    {
        let gauge_func = Gauge::with_opts_and_function(opts, function).unwrap();
        if let Some(ref registry) = self.registry {
            registry.register(Box::new(gauge_func.clone())).unwrap();
        }
        gauge_func
    }

    pub fn new_summary(&self, opts: SummaryOpts) -> Summary {
        let summary = Summary::with_opts(opts).unwrap();
        if let Some(ref registry) = self.registry {
            registry.register(Box::new(summary.clone())).unwrap();
        }
        summary
    }

    pub fn new_summary_vec(&self, opts: SummaryOpts, label_names: &[&str]) -> SummaryVec {
        let summary_vec = SummaryVec::new(opts, label_names).unwrap();
        if let Some(ref registry) = self.registry {
            registry.register(Box::new(summary_vec.clone())).unwrap();
        }
        summary_vec
    }

    pub fn new_histogram(&self, opts: HistogramOpts) -> Histogram {
        let histogram = Histogram::with_opts(opts).unwrap();
        if let Some(ref registry) = self.registry {
            registry.register(Box::new(histogram.clone())).unwrap();
        }
        histogram
    }

    pub fn new_histogram_vec(&self, opts: HistogramOpts, label_names: &[&str]) -> HistogramVec {
        let histogram_vec = HistogramVec::new(opts, label_names).unwrap();
        if let Some(ref registry) = self.registry {
            registry.register(Box::new(histogram_vec.clone())).unwrap();
        }
        histogram_vec
    }

    pub fn new_untyped_func<F>(&self, opts: UntypedOpts, function: F) -> Untyped
    where
        F: Fn() -> f64 + 'static,
    {
        let untyped_func = Untyped::with_opts_and_function(opts, function).unwrap();
        if let Some(ref registry) = self.registry {
            registry.register(Box::new(untyped_func.clone())).unwrap();
        }
        untyped_func
    }
}