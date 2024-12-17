use log::{info, warn};
// use prometheus::, IntCounterVec, Opts, Desc, proto::MetricFamily};
use crate::prometheus::encoder::Encoder;
use protobuf::Message;
use std::collections::HashMap;
use std::error::Error;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::task;
use xxhash_rust::xxh3::xxh3_64;

// TextEncoder

#[derive(Debug)]
struct Desc {
    fq_name: String,
    help: String,
    const_label_pairs: Vec<prometheus::proto::LabelPair>,
    variable_labels: Vec<String>,
    id: u64,
    dim_hash: u64,
    err: Option<String>,
}

impl Desc {
    fn new(
        fq_name: &str,
        help: &str,
        variable_labels: Vec<String>,
        const_labels: HashMap<String, String>,
    ) -> Self {
        let mut d = Desc {
            fq_name: fq_name.to_string(),
            help: help.to_string(),
            const_label_pairs: Vec::new(),
            variable_labels: variable_labels.clone(),
            id: 0,
            dim_hash: 0,
            err: None,
        };

        if !prometheus::is_valid_metric_name(fq_name) {
            d.err = Some(format!("{} is not a valid metric name", fq_name));
            return d;
        }

        let mut label_values = vec![fq_name.to_string()];
        let mut label_names = Vec::new();
        let mut label_name_set = HashMap::new();

        for (label_name, label_value) in &const_labels {
            if !prometheus::is_valid_label_name(label_name) {
                d.err = Some(format!(
                    "{} is not a valid label name for metric {}",
                    label_name, fq_name
                ));
                return d;
            }
            label_names.push(label_name.clone());
            label_name_set.insert(label_name.clone(), ());
            label_values.push(label_value.clone());
        }

        label_names.sort();

        for label in &variable_labels {
            if !prometheus::is_valid_label_name(label) {
                d.err = Some(format!(
                    "{} is not a valid label name for metric {}",
                    label, fq_name
                ));
                return d;
            }
            label_names.push(format!("${}", label));
            label_name_set.insert(format!("${}", label), ());
        }

        if label_names.len() != label_name_set.len() {
            d.err = Some(format!(
                "duplicate label names in constant and variable labels for metric {}",
                fq_name
            ));
            return d;
        }

        let mut hasher = xxh3_64::new();
        for val in &label_values {
            hasher.update(val.as_bytes());
            hasher.update(b"\x00");
        }
        d.id = hasher.digest();

        label_names.sort();
        hasher.reset();
        hasher.update(help.as_bytes());
        hasher.update(b"\x00");
        for label_name in &label_names {
            hasher.update(label_name.as_bytes());
            hasher.update(b"\x00");
        }
        d.dim_hash = hasher.digest();

        for (name, value) in const_labels {
            let mut label_pair = prometheus::proto::LabelPair::new();
            label_pair.set_name(name);
            label_pair.set_value(value);
            d.const_label_pairs.push(label_pair);
        }

        d.const_label_pairs
            .sort_by(|a, b| a.get_name().cmp(b.get_name()));

        d
    }

    fn new_invalid_desc(err: String) -> Self {
        Desc {
            fq_name: String::new(),
            help: String::new(),
            const_label_pairs: Vec::new(),
            variable_labels: Vec::new(),
            id: 0,
            dim_hash: 0,
            err: Some(err),
        }
    }

    fn to_string(&self) -> String {
        let lp_strings: Vec<String> = self
            .const_label_pairs
            .iter()
            .map(|lp| format!("{}={:?}", lp.get_name(), lp.get_value()))
            .collect();

        let vl_strings: Vec<String> = self
            .variable_labels
            .iter()
            .map(|vl| format!("${}", vl))
            .collect();

        format!(
            "Desc{{fq_name: {:?}, help: {:?}, const_labels: {{{}}}, variable_labels: {{{}}}}}",
            self.fq_name,
            self.help,
            lp_strings.join(","),
            vl_strings.join(","),
        )
    }
}

// fn main() {
//     let const_labels = HashMap::new();
//     let desc = Desc::new("test_metric", "This is a test metric", vec!["label1".to_string(), "label2".to_string()], const_labels);
//     println!("{}", desc.to_string());
// }
