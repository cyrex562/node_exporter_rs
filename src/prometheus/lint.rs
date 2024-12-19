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

use prometheus::{self, proto::MetricFamily, Registry, Gatherer};
use crate::promlint::{Linter, Problem};
use std::error::Error;

pub fn collect_and_lint(c: Box<dyn prometheus::core::Collector>, metric_names: &[&str]) -> Result<Vec<Problem>, Box<dyn Error>> {
    let reg = Registry::new_custom(Some("pedantic".to_string()), None)?;
    reg.register(c)?;

    gather_and_lint(&reg, metric_names)
}

pub fn gather_and_lint(g: &dyn Gatherer, metric_names: &[&str]) -> Result<Vec<Problem>, Box<dyn Error>> {
    let mut got = g.gather()?;
    if !metric_names.is_empty() {
        got = filter_metrics(got, metric_names);
    }
    Linter::new_with_metric_families(got).lint()
}

fn filter_metrics(metrics: Vec<MetricFamily>, metric_names: &[&str]) -> Vec<MetricFamily> {
    metrics.into_iter()
        .filter(|mf| metric_names.contains(&mf.get_name()))
        .collect()
}