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

use regex::Regex;

pub struct GoCollectorRule {
    matcher: Regex,
    deny: bool,
}

// GoCollectorOptions should not be used directly by anything, except the `collectors` module.
// Use it via the collectors module instead. See issue
// https://github.com/prometheus/client_golang/issues/1030.
//
// This is internal, so external users can only use it via `collector::with_go_collector_*` methods.
pub struct GoCollectorOptions {
    disable_mem_stats_like_metrics: bool,
    runtime_metric_sum_for_hist: std::collections::HashMap<String, String>,
    runtime_metric_rules: Vec<GoCollectorRule>,
}

lazy_static::lazy_static! {
    pub static ref GO_COLLECTOR_DEFAULT_RUNTIME_METRICS: Regex = Regex::new(r"/gc/gogc:percent|/gc/gomemlimit:bytes|/sched/gomaxprocs:threads").unwrap();
}