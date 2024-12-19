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
use regex::Regex;
use std::error::Error;

lazy_static::lazy_static! {
    static ref CAMEL_CASE: Regex = Regex::new(r"[a-z][A-Z]").unwrap();
    static ref UNIT_ABBREVIATIONS: Vec<&'static str> = vec!["kb", "mb", "gb", "tb", "pb", "eb", "kib", "mib", "gib", "tib", "pib", "eib"];
}

pub fn lint_metric_units(mf: &MetricFamily) -> Vec<Box<dyn Error>> {
    let mut problems = Vec::new();

    if let Some((unit, base)) = metric_units(mf.get_name()) {
        if unit != base {
            problems.push(format!("use base unit '{}' instead of '{}'", base, unit).into());
        }
    }

    problems
}

pub fn lint_metric_type_in_name(mf: &MetricFamily) -> Vec<Box<dyn Error>> {
    if mf.get_field_type() == MetricType::UNTYPED {
        return Vec::new();
    }

    let mut problems = Vec::new();
    let name = mf.get_name().to_lowercase();
    let typename = format!("{:?}", mf.get_field_type()).to_lowercase();

    if name.contains(&format!("_{}_", typename)) || name.ends_with(&format!("_{}", typename)) {
        problems.push(format!("metric name should not include type '{}'", typename).into());
    }

    problems
}

pub fn lint_reserved_chars(mf: &MetricFamily) -> Vec<Box<dyn Error>> {
    let mut problems = Vec::new();
    if mf.get_name().contains(':') {
        problems.push("metric names should not contain ':'".into());
    }
    problems
}

pub fn lint_camel_case(mf: &MetricFamily) -> Vec<Box<dyn Error>> {
    let mut problems = Vec::new();
    if CAMEL_CASE.is_match(mf.get_name()) {
        problems.push("metric names should be written in 'snake_case' not 'camelCase'".into());
    }

    for m in mf.get_metric() {
        for l in m.get_label() {
            if CAMEL_CASE.is_match(l.get_name()) {
                problems.push("label names should be written in 'snake_case' not 'camelCase'".into());
            }
        }
    }
    problems
}

pub fn lint_unit_abbreviations(mf: &MetricFamily) -> Vec<Box<dyn Error>> {
    let mut problems = Vec::new();
    let name = mf.get_name().to_lowercase();
    for &abbr in UNIT_ABBREVIATIONS.iter() {
        if name.contains(&format!("_{}_", abbr)) || name.ends_with(&format!("_{}", abbr)) {
            problems.push("metric names should not contain abbreviated units".into());
        }
    }
    problems
}

fn metric_units(name: &str) -> Option<(&str, &str)> {
    // This function should return the unit and base unit if applicable.
    // Placeholder implementation:
    if name.ends_with("_seconds") {
        Some(("seconds", "seconds"))
    } else if name.ends_with("_ms") {
        Some(("ms", "seconds"))
    } else {
        None
    }
}