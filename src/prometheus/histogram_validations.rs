// Copyright 2020 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use prometheus_client_model::{MetricFamily, MetricType};
use std::error::Error;

pub fn lint_histogram_summary_reserved(mf: &MetricFamily) -> Vec<Box<dyn Error>> {
    let mut problems = Vec::new();

    // These rules do not apply to untyped metrics.
    let t = mf.get_field_type();
    if t == MetricType::UNTYPED {
        return problems;
    }

    let is_histogram = t == MetricType::HISTOGRAM;
    let is_summary = t == MetricType::SUMMARY;

    let name = mf.get_name();

    if !is_histogram && name.ends_with("_bucket") {
        problems.push("non-histogram metrics should not have \"_bucket\" suffix".into());
    }
    if !is_histogram && !is_summary && name.ends_with("_count") {
        problems.push("non-histogram and non-summary metrics should not have \"_count\" suffix".into());
    }
    if !is_histogram && !is_summary && name.ends_with("_sum") {
        problems.push("non-histogram and non-summary metrics should not have \"_sum\" suffix".into());
    }

    for m in mf.get_metric() {
        for l in m.get_label() {
            let ln = l.get_name();

            if !is_histogram && ln == "le" {
                problems.push("non-histogram metrics should not have \"le\" label".into());
            }
            if !is_summary && ln == "quantile" {
                problems.push("non-summary metrics should not have \"quantile\" label".into());
            }
        }
    }

    problems
}