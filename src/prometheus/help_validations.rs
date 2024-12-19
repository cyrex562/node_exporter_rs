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
use std::error::Error;

pub fn lint_help(mf: &MetricFamily) -> Vec<Box<dyn Error>> {
    let mut problems = Vec::new();

    // Expect all metrics to have help text available.
    if mf.get_help().is_empty() {
        problems.push("no help text".into());
    }

    problems
}