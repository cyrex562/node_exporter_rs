// Copyright 2015 The Prometheus Authors
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

// Package collector includes all individual collectors to gather and export system metrics.
package collector

import (
	"errors"
	"fmt"
	"log/slog"
	"sync"
	"time"

	"github.com/alecthomas/kingpin/v2"
	"github.com/prometheus/client_golang/prometheus"
)

// Namespace defines the common namespace to be used by all metrics.
const namespace = "node"

var (
	scrapeDurationDesc = prometheus.NewDesc(
		prometheus.BuildFQName(namespace, "scrape", "collector_duration_seconds"),
		"node_exporter: Duration of a collector scrape.",
		[]string{"collector"},
		nil,
	)
	scrapeSuccessDesc = prometheus.NewDesc(
		prometheus.BuildFQName(namespace, "scrape", "collector_success"),
		"node_exporter: Whether a collector succeeded.",
		[]string{"collector"},
		nil,
	)
)

const (
	defaultEnabled  = true
	defaultDisabled = false
)

var (
	factories              = make(map[string]func(logger *slog.Logger) (Collector, error))
	initiatedCollectorsMtx = sync.Mutex{}
	initiatedCollectors    = make(map[string]Collector)
	collectorState         = make(map[string]*bool)
	forcedCollectors       = map[string]bool{} // collectors which have been explicitly enabled or disabled
)

func registerCollector(collector string, isDefaultEnabled bool, factory func(logger *slog.Logger) (Collector, error)) {
	var helpDefaultState string
	if isDefaultEnabled {
		helpDefaultState = "enabled"
	} else {
		helpDefaultState = "disabled"
	}

	flagName := fmt.Sprintf("collector.%s", collector)
	flagHelp := fmt.Sprintf("Enable the %s collector (default: %s).", collector, helpDefaultState)
	defaultValue := fmt.Sprintf("%v", isDefaultEnabled)

	flag := kingpin.Flag(flagName, flagHelp).Default(defaultValue).Action(collectorFlagAction(collector)).Bool()
	collectorState[collector] = flag

	factories[collector] = factory
}

// NodeCollector implements the prometheus.Collector interface.
type NodeCollector struct {
	Collectors map[string]Collector
	logger     *slog.Logger
}

// DisableDefaultCollectors sets the collector state to false for all collectors which
// have not been explicitly enabled on the command line.
func DisableDefaultCollectors() {
	for c := range collectorState {
		if _, ok := forcedCollectors[c]; !ok {
			*collectorState[c] = false
		}
	}
}

// collectorFlagAction generates a new action function for the given collector
// to track whether it has been explicitly enabled or disabled from the command line.
// A new action function is needed for each collector flag because the ParseContext
// does not contain information about which flag called the action.
// See: https://github.com/alecthomas/kingpin/issues/294
func collectorFlagAction(collector string) func(ctx *kingpin.ParseContext) error {
	return func(ctx *kingpin.ParseContext) error {
		forcedCollectors[collector] = true
		return nil
	}
}

// NewNodeCollector creates a new NodeCollector.
func NewNodeCollector(logger *slog.Logger, filters ...string) (*NodeCollector, error) {
	f := make(map[string]bool)
	for _, filter := range filters {
		enabled, exist := collectorState[filter]
		if !exist {
			return nil, fmt.Errorf("missing collector: %s", filter)
		}
		if !*enabled {
			return nil, fmt.Errorf("disabled collector: %s", filter)
		}
		f[filter] = true
	}
	collectors := make(map[string]Collector)
	initiatedCollectorsMtx.Lock()
	defer initiatedCollectorsMtx.Unlock()
	for key, enabled := range collectorState {
		if !*enabled || (len(f) > 0 && !f[key]) {
			continue
		}
		if collector, ok := initiatedCollectors[key]; ok {
			collectors[key] = collector
		} else {
			collector, err := factories[key](logger.With("collector", key))
			if err != nil {
				return nil, err
			}
			collectors[key] = collector
			initiatedCollectors[key] = collector
		}
	}
	return &NodeCollector{Collectors: collectors, logger: logger}, nil
}

// Describe implements the prometheus.Collector interface.
func (n NodeCollector) Describe(ch chan<- *prometheus.Desc) {
	ch <- scrapeDurationDesc
	ch <- scrapeSuccessDesc
}

// Collect implements the prometheus.Collector interface.
func (n NodeCollector) Collect(ch chan<- prometheus.Metric) {
	wg := sync.WaitGroup{}
	wg.Add(len(n.Collectors))
	for name, c := range n.Collectors {
		go func(name string, c Collector) {
			execute(name, c, ch, n.logger)
			wg.Done()
		}(name, c)
	}
	wg.Wait()
}

func execute(name string, c Collector, ch chan<- prometheus.Metric, logger *slog.Logger) {
	begin := time.Now()
	err := c.Update(ch)
	duration := time.Since(begin)
	var success float64

	if err != nil {
		if IsNoDataError(err) {
			logger.Debug("collector returned no data", "name", name, "duration_seconds", duration.Seconds(), "err", err)
		} else {
			logger.Error("collector failed", "name", name, "duration_seconds", duration.Seconds(), "err", err)
		}
		success = 0
	} else {
		logger.Debug("collector succeeded", "name", name, "duration_seconds", duration.Seconds())
		success = 1
	}
	ch <- prometheus.MustNewConstMetric(scrapeDurationDesc, prometheus.GaugeValue, duration.Seconds(), name)
	ch <- prometheus.MustNewConstMetric(scrapeSuccessDesc, prometheus.GaugeValue, success, name)
}

// Collector is the interface a collector has to implement.
type Collector interface {
	// Get new metrics and expose them via prometheus registry.
	Update(ch chan<- prometheus.Metric) error
}

type typedDesc struct {
	desc      *prometheus.Desc
	valueType prometheus.ValueType
}

func (d *typedDesc) mustNewConstMetric(value float64, labels ...string) prometheus.Metric {
	return prometheus.MustNewConstMetric(d.desc, d.valueType, value, labels...)
}

// ErrNoData indicates the collector found no data to collect, but had no other error.
var ErrNoData = errors.New("collector returned no data")

func IsNoDataError(err error) bool {
	return err == ErrNoData
}

// pushMetric helps construct and convert a variety of value types into Prometheus float64 metrics.
func pushMetric(ch chan<- prometheus.Metric, fieldDesc *prometheus.Desc, name string, value interface{}, valueType prometheus.ValueType, labelValues ...string) {
	var fVal float64
	switch val := value.(type) {
	case uint8:
		fVal = float64(val)
	case uint16:
		fVal = float64(val)
	case uint32:
		fVal = float64(val)
	case uint64:
		fVal = float64(val)
	case int64:
		fVal = float64(val)
	case *uint8:
		if val == nil {
			return
		}
		fVal = float64(*val)
	case *uint16:
		if val == nil {
			return
		}
		fVal = float64(*val)
	case *uint32:
		if val == nil {
			return
		}
		fVal = float64(*val)
	case *uint64:
		if val == nil {
			return
		}
		fVal = float64(*val)
	case *int64:
		if val == nil {
			return
		}
		fVal = float64(*val)
	default:
		return
	}

	ch <- prometheus.MustNewConstMetric(fieldDesc, valueType, fVal, labelValues...)
}


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