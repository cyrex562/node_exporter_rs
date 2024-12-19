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
use crate::validations::{
    lint_help, lint_metric_units, lint_counter, lint_histogram_summary_reserved,
    lint_metric_type_in_name, lint_reserved_chars, lint_camel_case, lint_unit_abbreviations,
    lint_duplicate_metric,
};

pub type Validation = fn(&MetricFamily) -> Vec<Box<dyn std::error::Error>>;

pub fn default_validations() -> Vec<Validation> {
    vec![
        lint_help,
        lint_metric_units,
        lint_counter,
        lint_histogram_summary_reserved,
        lint_metric_type_in_name,
        lint_reserved_chars,
        lint_camel_case,
        lint_unit_abbreviations,
        lint_duplicate_metric,
    ]
}