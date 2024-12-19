// Copyright 2021 The Prometheus Authors
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

const DMI_CLASS_PATH: &str = "class/dmi/id";

#[derive(Debug, Default)]
pub struct DMIClass {
    bios_date: Option<String>,
    bios_release: Option<String>,
    bios_vendor: Option<String>,
    bios_version: Option<String>,
    board_asset_tag: Option<String>,
    board_name: Option<String>,
    board_serial: Option<String>,
    board_vendor: Option<String>,
    board_version: Option<String>,
    chassis_asset_tag: Option<String>,
    chassis_serial: Option<String>,
    chassis_type: Option<String>,
    chassis_vendor: Option<String>,
    chassis_version: Option<String>,
    product_family: Option<String>,
    product_name: Option<String>,
    product_serial: Option<String>,
    product_sku: Option<String>,
    product_uuid: Option<String>,
    product_version: Option<String>,
    system_vendor: Option<String>,
}

pub struct FS {
    sys_path: String,
}

impl FS {
    pub fn new(sys_path: String) -> Self {
        FS { sys_path }
    }

    pub fn dmi_class(&self) -> Result<DMIClass, io::Error> {
        let path = Path::new(&self.sys_path).join(DMI_CLASS_PATH);
        let entries = fs::read_dir(path)?;

        let mut dmi = DMIClass::default();
        for entry in entries {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }

            let name = entry.file_name().into_string().unwrap();
            if name == "modalias" || name == "uevent" {
                continue;
            }

            let filename = entry.path();
            let value = fs::read_to_string(&filename)?.trim().to_string();

            match name.as_str() {
                "bios_date" => dmi.bios_date = Some(value),
                "bios_release" => dmi.bios_release = Some(value),
                "bios_vendor" => dmi.bios_vendor = Some(value),
                "bios_version" => dmi.bios_version = Some(value),
                "board_asset_tag" => dmi.board_asset_tag = Some(value),
                "board_name" => dmi.board_name = Some(value),
                "board_serial" => dmi.board_serial = Some(value),
                "board_vendor" => dmi.board_vendor = Some(value),
                "board_version" => dmi.board_version = Some(value),
                "chassis_asset_tag" => dmi.chassis_asset_tag = Some(value),
                "chassis_serial" => dmi.chassis_serial = Some(value),
                "chassis_type" => dmi.chassis_type = Some(value),
                "chassis_vendor" => dmi.chassis_vendor = Some(value),
                "chassis_version" => dmi.chassis_version = Some(value),
                "product_family" => dmi.product_family = Some(value),
                "product_name" => dmi.product_name = Some(value),
                "product_serial" => dmi.product_serial = Some(value),
                "product_sku" => dmi.product_sku = Some(value),
                "product_uuid" => dmi.product_uuid = Some(value),
                "product_version" => dmi.product_version = Some(value),
                "sys_vendor" => dmi.system_vendor = Some(value),
                _ => {}
            }
        }

        Ok(dmi)
    }
}