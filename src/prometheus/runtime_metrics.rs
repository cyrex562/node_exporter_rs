// Copyright 2021 The Prometheus Authors
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

#[cfg(target_os = "linux")]
use std::path::Path;
use std::collections::HashMap;
use std::f64;
use std::path::PathBuf;
use std::str::FromStr;

use prometheus::core::Collector;
use prometheus::proto::MetricFamily;
use prometheus::{Opts, Registry};
use regex::Regex;

pub struct GoCollectorRule {
    matcher: Regex,
    deny: bool,
}

pub struct GoCollectorOptions {
    disable_mem_stats_like_metrics: bool,
    runtime_metric_sum_for_hist: HashMap<String, String>,
    runtime_metric_rules: Vec<GoCollectorRule>,
}

pub fn runtime_metrics_to_prom(d: &metrics::Description) -> (String, String, String, bool) {
    let namespace = "go".to_string();

    let comp: Vec<&str> = d.name.splitn(2, ':').collect();
    let key = comp[0];
    let unit = comp[1];

    let subsystem = key[1..].replace("/", "_").replace("-", "_");
    let mut name = key.split('/').last().unwrap().replace("-", "_");
    name.push('_');
    name.push_str(&unit.replace("-", "_").replace("*", "_").replace("/", "_per_"));
    if d.cumulative && d.kind != metrics::Kind::Float64Histogram {
        name.push_str("_total");
    }

    let valid = prometheus::core::is_valid_metric_name(&format!("{}_{}_{}", namespace, subsystem, name));
    let valid = match d.kind {
        metrics::Kind::Uint64 | metrics::Kind::Float64 | metrics::Kind::Float64Histogram => valid,
        _ => false,
    };

    (namespace, subsystem, name, valid)
}

pub fn runtime_metrics_buckets_for_unit(buckets: Vec<f64>, unit: &str) -> Vec<f64> {
    match unit {
        "bytes" => re_bucket_exp(buckets, 2.0),
        "seconds" => {
            let mut b = re_bucket_exp(buckets, 10.0);
            for i in 0..b.len() {
                if b[i] > 1.0 {
                    b[i] = f64::INFINITY;
                    b.truncate(i + 1);
                    break;
                }
            }
            b
        }
        _ => buckets,
    }
}

fn re_bucket_exp(mut buckets: Vec<f64>, base: f64) -> Vec<f64> {
    let mut new_buckets = Vec::new();
    let mut bucket = buckets[0];

    if bucket == f64::NEG_INFINITY {
        new_buckets.push(bucket);
        buckets.remove(0);
        bucket = buckets[0];
    }

    for i in 1..buckets.len() {
        if (bucket >= 0.0 && buckets[i] < bucket * base) || (bucket < 0.0 && buckets[i] < bucket / base) {
            continue;
        }
        new_buckets.push(bucket);
        bucket = buckets[i];
    }
    new_buckets.push(bucket);
    new_buckets
}
