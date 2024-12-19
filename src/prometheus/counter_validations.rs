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

use prometheus_client_model::MetricFamily;
use prometheus_client_model::MetricType;
use std::error::Error;

pub fn lint_counter(mf: &MetricFamily) -> Vec<Box<dyn Error>> {
    let mut problems = Vec::new();

    let is_counter = mf.get_field_type() == MetricType::COUNTER;
    let is_untyped = mf.get_field_type() == MetricType::UNTYPED;
    let has_total_suffix = mf.get_name().ends_with("_total");

    match (is_counter, is_untyped, has_total_suffix) {
        (true, _, false) => problems.push("counter metrics should have \"_total\" suffix".into()),
        (false, false, true) => problems.push("non-counter metrics should not have \"_total\" suffix".into()),
        _ => {}
    }

    problems
}