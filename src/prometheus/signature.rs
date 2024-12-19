use std::collections::HashMap;
use std::collections::HashSet;

const SEPARATOR_BYTE: u8 = 255;
const EMPTY_LABEL_SIGNATURE: u64 = hash_new();

pub fn labels_to_signature(labels: &HashMap<String, String>) -> u64 {
    if labels.is_empty() {
        return EMPTY_LABEL_SIGNATURE;
    }

    let mut label_names: Vec<&String> = labels.keys().collect();
    label_names.sort();

    let mut sum = hash_new();
    for label_name in label_names {
        sum = hash_add(sum, label_name);
        sum = hash_add_byte(sum, SEPARATOR_BYTE);
        sum = hash_add(sum, &labels[label_name]);
        sum = hash_add_byte(sum, SEPARATOR_BYTE);
    }
    sum
}

pub fn label_set_to_fingerprint(ls: &LabelSet) -> Fingerprint {
    if ls.is_empty() {
        return Fingerprint(EMPTY_LABEL_SIGNATURE);
    }

    let mut label_names: Vec<&LabelName> = ls.keys().collect();
    label_names.sort();

    let mut sum = hash_new();
    for label_name in label_names {
        sum = hash_add(sum, label_name.as_str());
        sum = hash_add_byte(sum, SEPARATOR_BYTE);
        sum = hash_add(sum, ls.get(label_name).unwrap().as_str());
        sum = hash_add_byte(sum, SEPARATOR_BYTE);
    }
    Fingerprint(sum)
}

pub fn label_set_to_fast_fingerprint(ls: &LabelSet) -> Fingerprint {
    if ls.is_empty() {
        return Fingerprint(EMPTY_LABEL_SIGNATURE);
    }

    let mut result = 0;
    for (label_name, label_value) in ls {
        let mut sum = hash_new();
        sum = hash_add(sum, label_name.as_str());
        sum = hash_add_byte(sum, SEPARATOR_BYTE);
        sum = hash_add(sum, label_value.as_str());
        result ^= sum;
    }
    Fingerprint(result)
}

pub fn signature_for_labels(m: &Metric, labels: &[LabelName]) -> u64 {
    if labels.is_empty() {
        return EMPTY_LABEL_SIGNATURE;
    }

    let mut sorted_labels = labels.to_vec();
    sorted_labels.sort();

    let mut sum = hash_new();
    for label in sorted_labels {
        sum = hash_add(sum, label.as_str());
        sum = hash_add_byte(sum, SEPARATOR_BYTE);
        sum = hash_add(sum, m.get(label).unwrap().as_str());
        sum = hash_add_byte(sum, SEPARATOR_BYTE);
    }
    sum
}

pub fn signature_without_labels(m: &Metric, labels: &HashSet<LabelName>) -> u64 {
    if m.is_empty() {
        return EMPTY_LABEL_SIGNATURE;
    }

    let mut label_names: Vec<&LabelName> = m.keys().filter(|&k| !labels.contains(k)).collect();
    if label_names.is_empty() {
        return EMPTY_LABEL_SIGNATURE;
    }
    label_names.sort();

    let mut sum = hash_new();
    for label_name in label_names {
        sum = hash_add(sum, label_name.as_str());
        sum = hash_add_byte(sum, SEPARATOR_BYTE);
        sum = hash_add(sum, m.get(label_name).unwrap().as_str());
        sum = hash_add_byte(sum, SEPARATOR_BYTE);
    }
    sum
}