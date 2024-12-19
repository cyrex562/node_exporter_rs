// Copyright 2018 The Prometheus Authors
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

//! Package testutil provides helpers to test code using the prometheus crate.

use prometheus::{self, proto::MetricFamily, Encoder, TextEncoder, Registry, Gatherer};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::sync::mpsc::channel;

pub fn to_float64(c: Box<dyn prometheus::core::Collector>) -> f64 {
    let (tx, rx) = channel();
    c.collect(Box::new(tx));
    let metrics: Vec<_> = rx.into_iter().collect();

    if metrics.len() != 1 {
        panic!("collected {} metrics instead of exactly 1", metrics.len());
    }

    let metric = metrics.into_iter().next().unwrap();
    let mut pb = MetricFamily::default();
    metric.write(&mut pb).unwrap();

    if let Some(gauge) = pb.get_gauge() {
        return gauge.get_value();
    }
    if let Some(counter) = pb.get_counter() {
        return counter.get_value();
    }
    if let Some(untyped) = pb.get_untyped() {
        return untyped.get_value();
    }
    panic!("collected a non-gauge/counter/untyped metric");
}

pub fn collect_and_count(c: Box<dyn prometheus::core::Collector>, metric_names: &[&str]) -> usize {
    let reg = Registry::new();
    reg.register(c).unwrap();
    gather_and_count(&reg, metric_names).unwrap()
}

pub fn gather_and_count(g: &dyn Gatherer, metric_names: &[&str]) -> Result<usize, Box<dyn Error>> {
    let mut got = g.gather()?;
    if !metric_names.is_empty() {
        got = filter_metrics(got, metric_names);
    }

    Ok(got.iter().map(|mf| mf.get_metric().len()).sum())
}

pub fn scrape_and_compare(url: &str, expected: &mut dyn Read, metric_names: &[&str]) -> Result<(), Box<dyn Error>> {
    let mut resp = reqwest::blocking::get(url)?;
    if resp.status() != reqwest::StatusCode::OK {
        return Err(format!("the scraping target returned a status code other than 200: {}", resp.status()).into());
    }

    let mut scraped = Vec::new();
    resp.copy_to(&mut scraped)?;

    let mut expected_buf = Vec::new();
    expected.read_to_end(&mut expected_buf)?;

    compare_metric_families(&scraped, &expected_buf, metric_names)
}

pub fn collect_and_compare(c: Box<dyn prometheus::core::Collector>, expected: &mut dyn Read, metric_names: &[&str]) -> Result<(), Box<dyn Error>> {
    let reg = Registry::new();
    reg.register(c).unwrap();
    gather_and_compare(&reg, expected, metric_names)
}

pub fn gather_and_compare(g: &dyn Gatherer, expected: &mut dyn Read, metric_names: &[&str]) -> Result<(), Box<dyn Error>> {
    let got = g.gather()?;
    let mut expected_buf = Vec::new();
    expected.read_to_end(&mut expected_buf)?;

    compare_metric_families(&got, &expected_buf, metric_names)
}

pub fn collect_and_format(c: Box<dyn prometheus::core::Collector>, format: &str, metric_names: &[&str]) -> Result<Vec<u8>, Box<dyn Error>> {
    let reg = Registry::new();
    reg.register(c).unwrap();

    let mut got_filtered = reg.gather()?;
    got_filtered = filter_metrics(got_filtered, metric_names);

    let mut got_formatted = Vec::new();
    let encoder = match format {
        "text" => TextEncoder::new(),
        _ => return Err("unsupported format".into()),
    };
    encoder.encode(&got_filtered, &mut got_formatted)?;

    Ok(got_formatted)
}

fn filter_metrics(metrics: Vec<MetricFamily>, names: &[&str]) -> Vec<MetricFamily> {
    metrics.into_iter()
        .filter(|m| names.contains(&m.get_name()))
        .collect()
}

fn compare_metric_families(got: &[u8], expected: &[u8], metric_names: &[&str]) -> Result<(), Box<dyn Error>> {
    let got_families = parse_metric_families(got)?;
    let expected_families = parse_metric_families(expected)?;

    let got_filtered = filter_metrics(got_families, metric_names);
    let expected_filtered = filter_metrics(expected_families, metric_names);

    if got_filtered != expected_filtered {
        return Err(format!("metrics do not match: got {:?}, expected {:?}", got_filtered, expected_filtered).into());
    }

    Ok(())
}

fn parse_metric_families(data: &[u8]) -> Result<Vec<MetricFamily>, Box<dyn Error>> {
    let mut parser = TextEncoder::new();
    let families = parser.decode(data)?;
    Ok(families)
}