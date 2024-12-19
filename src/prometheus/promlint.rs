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

//! Package promlint provides a linter for Prometheus metrics.

use prometheus_client_model::MetricFamily;
use prometheus_client_model::MetricType;
use prometheus::proto::MetricFamily as ProtoMetricFamily;
use prometheus::proto::MetricType as ProtoMetricType;
use std::cmp::Ordering;
use std::error::Error;
use std::io::{self, Read};
use std::str::FromStr;

pub struct Problem {
    pub metric: String,
    pub text: String,
}

impl Problem {
    pub fn new(metric: &str, text: &str) -> Self {
        Problem {
            metric: metric.to_string(),
            text: text.to_string(),
        }
    }
}

pub type Validation = fn(&MetricFamily) -> Vec<Box<dyn Error>>;

pub struct Linter<R: Read> {
    reader: Option<R>,
    mfs: Vec<ProtoMetricFamily>,
    custom_validations: Vec<Validation>,
}

impl<R: Read> Linter<R> {
    pub fn new(reader: R) -> Self {
        Linter {
            reader: Some(reader),
            mfs: Vec::new(),
            custom_validations: Vec::new(),
        }
    }

    pub fn new_with_metric_families(mfs: Vec<ProtoMetricFamily>) -> Self {
        Linter {
            reader: None,
            mfs,
            custom_validations: Vec::new(),
        }
    }

    pub fn add_custom_validations(&mut self, validations: Vec<Validation>) {
        self.custom_validations.extend(validations);
    }

    pub fn lint(&mut self) -> Result<Vec<Problem>, Box<dyn Error>> {
        let mut problems = Vec::new();

        if let Some(reader) = &mut self.reader {
            let mut decoder = prometheus::TextDecoder::new(reader);

            while let Some(mf) = decoder.next()? {
                problems.extend(self.lint_metric_family(&mf));
            }
        }

        for mf in &self.mfs {
            problems.extend(self.lint_metric_family(mf));
        }

        problems.sort_by(|a, b| {
            if a.metric == b.metric {
                a.text.cmp(&b.text)
            } else {
                a.metric.cmp(&b.metric)
            }
        });

        Ok(problems)
    }

    fn lint_metric_family(&self, mf: &ProtoMetricFamily) -> Vec<Problem> {
        let mut problems = Vec::new();

        for validation in &default_validations() {
            for err in validation(mf) {
                problems.push(Problem::new(mf.get_name(), &err.to_string()));
            }
        }

        for validation in &self.custom_validations {
            for err in validation(mf) {
                problems.push(Problem::new(mf.get_name(), &err.to_string()));
            }
        }

        problems
    }
}

fn default_validations() -> Vec<Validation> {
    vec![
        // Add default validation functions here
    ]
}