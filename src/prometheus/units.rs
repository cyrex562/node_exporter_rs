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

use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref UNITS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Base units.
        m.insert("amperes", "amperes");
        m.insert("bytes", "bytes");
        m.insert("celsius", "celsius");
        m.insert("grams", "grams");
        m.insert("joules", "joules");
        m.insert("kelvin", "kelvin");
        m.insert("meters", "meters");
        m.insert("metres", "metres");
        m.insert("seconds", "seconds");
        m.insert("volts", "volts");
        // Non base units.
        // Time.
        m.insert("minutes", "seconds");
        m.insert("hours", "seconds");
        m.insert("days", "seconds");
        m.insert("weeks", "seconds");
        // Temperature.
        m.insert("kelvins", "kelvin");
        m.insert("fahrenheit", "celsius");
        m.insert("rankine", "celsius");
        // Length.
        m.insert("inches", "meters");
        m.insert("yards", "meters");
        m.insert("miles", "meters");
        // Bytes.
        m.insert("bits", "bytes");
        // Energy.
        m.insert("calories", "joules");
        // Mass.
        m.insert("pounds", "grams");
        m.insert("ounces", "grams");
        m
    };

    static ref UNIT_PREFIXES: Vec<&'static str> = vec![
        "pico", "nano", "micro", "milli", "centi", "deci", "deca", "hecto", "kilo", "kibi",
        "mega", "mibi", "giga", "gibi", "tera", "tebi", "peta", "pebi"
    ];

    static ref UNIT_ABBREVIATIONS: Vec<&'static str> = vec![
        "s", "ms", "us", "ns", "sec", "b", "kb", "mb", "gb", "tb", "pb", "m", "h", "d"
    ];
}

pub fn metric_units(m: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = m.split('_').collect();

    for part in parts {
        if let Some(&base) = UNITS.get(part) {
            return Some((part, base));
        }

        for &prefix in UNIT_PREFIXES.iter() {
            if part.starts_with(prefix) {
                if let Some(&base) = UNITS.get(&part[prefix.len()..]) {
                    return Some((part, base));
                }
            }
        }
    }

    None
}