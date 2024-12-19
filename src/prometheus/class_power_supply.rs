// Copyright 2018 The Prometheus Authors
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

use std::fs;
use std::io;
use std::path::Path;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct PowerSupply {
    name: String,
    authentic: Option<i64>,
    calibrate: Option<i64>,
    capacity: Option<i64>,
    capacity_alert_max: Option<i64>,
    capacity_alert_min: Option<i64>,
    capacity_level: Option<String>,
    charge_avg: Option<i64>,
    charge_control_limit: Option<i64>,
    charge_control_limit_max: Option<i64>,
    charge_counter: Option<i64>,
    charge_empty: Option<i64>,
    charge_empty_design: Option<i64>,
    charge_full: Option<i64>,
    charge_full_design: Option<i64>,
    charge_now: Option<i64>,
    charge_term_current: Option<i64>,
    charge_type: Option<String>,
    constant_charge_current: Option<i64>,
    constant_charge_current_max: Option<i64>,
    constant_charge_voltage: Option<i64>,
    constant_charge_voltage_max: Option<i64>,
    current_avg: Option<i64>,
    current_boot: Option<i64>,
    current_max: Option<i64>,
    current_now: Option<i64>,
    cycle_count: Option<i64>,
    energy_avg: Option<i64>,
    energy_empty: Option<i64>,
    energy_empty_design: Option<i64>,
    energy_full: Option<i64>,
    energy_full_design: Option<i64>,
    energy_now: Option<i64>,
    health: Option<String>,
    input_current_limit: Option<i64>,
    manufacturer: Option<String>,
    model_name: Option<String>,
    online: Option<i64>,
    power_avg: Option<i64>,
    power_now: Option<i64>,
    precharge_current: Option<i64>,
    present: Option<i64>,
    scope: Option<String>,
    serial_number: Option<String>,
    status: Option<String>,
    technology: Option<String>,
    temp: Option<i64>,
    temp_alert_max: Option<i64>,
    temp_alert_min: Option<i64>,
    temp_ambient: Option<i64>,
    temp_ambient_max: Option<i64>,
    temp_ambient_min: Option<i64>,
    temp_max: Option<i64>,
    temp_min: Option<i64>,
    time_to_empty_avg: Option<i64>,
    time_to_empty_now: Option<i64>,
    time_to_full_avg: Option<i64>,
    time_to_full_now: Option<i64>,
    type_: Option<String>,
    usb_type: Option<String>,
    voltage_avg: Option<i64>,
    voltage_boot: Option<i64>,
    voltage_max: Option<i64>,
    voltage_max_design: Option<i64>,
    voltage_min: Option<i64>,
    voltage_min_design: Option<i64>,
    voltage_now: Option<i64>,
    voltage_ocv: Option<i64>,
}

pub type PowerSupplyClass = HashMap<String, PowerSupply>;

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn power_supply_class(&self) -> Result<PowerSupplyClass, io::Error> {
        let path = Path::new(&self.sys_path).join("class/power_supply");
        let dirs = fs::read_dir(path)?;

        let mut psc = PowerSupplyClass::new();
        for dir in dirs {
            let dir = dir?;
            let ps = parse_power_supply(&dir.path())?;
            psc.insert(ps.name.clone(), ps);
        }

        Ok(psc)
    }
}

fn parse_power_supply(path: &Path) -> Result<PowerSupply, io::Error> {
    let files = fs::read_dir(path)?;

    let mut ps = PowerSupply::default();
    for file in files {
        let file = file?;
        if !file.file_type()?.is_file() {
            continue;
        }

        let name = file.file_name().to_string_lossy().to_string();
        let value = fs::read_to_string(file.path())?.trim().to_string();

        match name.as_str() {
            "authentic" => ps.authentic = value.parse().ok(),
            "calibrate" => ps.calibrate = value.parse().ok(),
            "capacity" => ps.capacity = value.parse().ok(),
            "capacity_alert_max" => ps.capacity_alert_max = value.parse().ok(),
            "capacity_alert_min" => ps.capacity_alert_min = value.parse().ok(),
            "capacity_level" => ps.capacity_level = Some(value),
            "charge_avg" => ps.charge_avg = value.parse().ok(),
            "charge_control_limit" => ps.charge_control_limit = value.parse().ok(),
            "charge_control_limit_max" => ps.charge_control_limit_max = value.parse().ok(),
            "charge_counter" => ps.charge_counter = value.parse().ok(),
            "charge_empty" => ps.charge_empty = value.parse().ok(),
            "charge_empty_design" => ps.charge_empty_design = value.parse().ok(),
            "charge_full" => ps.charge_full = value.parse().ok(),
            "charge_full_design" => ps.charge_full_design = value.parse().ok(),
            "charge_now" => ps.charge_now = value.parse().ok(),
            "charge_term_current" => ps.charge_term_current = value.parse().ok(),
            "charge_type" => ps.charge_type = Some(value),
            "constant_charge_current" => ps.constant_charge_current = value.parse().ok(),
            "constant_charge_current_max" => ps.constant_charge_current_max = value.parse().ok(),
            "constant_charge_voltage" => ps.constant_charge_voltage = value.parse().ok(),
            "constant_charge_voltage_max" => ps.constant_charge_voltage_max = value.parse().ok(),
            "current_avg" => ps.current_avg = value.parse().ok(),
            "current_boot" => ps.current_boot = value.parse().ok(),
            "current_max" => ps.current_max = value.parse().ok(),
            "current_now" => ps.current_now = value.parse().ok(),
            "cycle_count" => ps.cycle_count = value.parse().ok(),
            "energy_avg" => ps.energy_avg = value.parse().ok(),
            "energy_empty" => ps.energy_empty = value.parse().ok(),
            "energy_empty_design" => ps.energy_empty_design = value.parse().ok(),
            "energy_full" => ps.energy_full = value.parse().ok(),
            "energy_full_design" => ps.energy_full_design = value.parse().ok(),
            "energy_now" => ps.energy_now = value.parse().ok(),
            "health" => ps.health = Some(value),
            "input_current_limit" => ps.input_current_limit = value.parse().ok(),
            "manufacturer" => ps.manufacturer = Some(value),
            "model_name" => ps.model_name = Some(value),
            "online" => ps.online = value.parse().ok(),
            "power_avg" => ps.power_avg = value.parse().ok(),
            "power_now" => ps.power_now = value.parse().ok(),
            "precharge_current" => ps.precharge_current = value.parse().ok(),
            "present" => ps.present = value.parse().ok(),
            "scope" => ps.scope = Some(value),
            "serial_number" => ps.serial_number = Some(value),
            "status" => ps.status = Some(value),
            "technology" => ps.technology = Some(value),
            "temp" => ps.temp = value.parse().ok(),
            "temp_alert_max" => ps.temp_alert_max = value.parse().ok(),
            "temp_alert_min" => ps.temp_alert_min = value.parse().ok(),
            "temp_ambient" => ps.temp_ambient = value.parse().ok(),
            "temp_ambient_max" => ps.temp_ambient_max = value.parse().ok(),
            "temp_ambient_min" => ps.temp_ambient_min = value.parse().ok(),
            "temp_max" => ps.temp_max = value.parse().ok(),
            "temp_min" => ps.temp_min = value.parse().ok(),
            "time_to_empty_avg" => ps.time_to_empty_avg = value.parse().ok(),
            "time_to_empty_now" => ps.time_to_empty_now = value.parse().ok(),
            "time_to_full_avg" => ps.time_to_full_avg = value.parse().ok(),
            "time_to_full_now" => ps.time_to_full_now = value.parse().ok(),
            "type" => ps.type_ = Some(value),
            "usb_type" => ps.usb_type = Some(value),
            "voltage_avg" => ps.voltage_avg = value.parse().ok(),
            "voltage_boot" => ps.voltage_boot = value.parse().ok(),
            "voltage_max" => ps.voltage_max = value.parse().ok(),
            "voltage_max_design" => ps.voltage_max_design = value.parse().ok(),
            "voltage_min" => ps.voltage_min = value.parse().ok(),
            "voltage_min_design" => ps.voltage_min_design = value.parse().ok(),
            "voltage_now" => ps.voltage_now = value.parse().ok(),
            "voltage_ocv" => ps.voltage_ocv = value.parse().ok(),
            _ => {}
        }
    }

    Ok(ps)
}