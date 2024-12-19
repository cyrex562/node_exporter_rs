// Copyright 2020 The Prometheus Authors
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

use clap::{App, Arg};
use std::env;

pub struct FlagConfig {
    pub web_listen_addresses: Vec<String>,
    pub web_systemd_socket: bool,
    pub web_config_file: String,
}

pub fn add_flags<'a>(app: App<'a>, default_address: &str) -> App<'a> {
    let systemd_socket = if cfg!(target_os = "linux") {
        true
    } else {
        false
    };

    app.arg(
        Arg::with_name("web.listen-address")
            .long("web.listen-address")
            .help("Addresses on which to expose metrics and web interface. Repeatable for multiple addresses. Examples: `:9100` or `[::1]:9100` for http, `vsock://:9100` for vsock")
            .default_value(default_address)
            .multiple(true)
            .takes_value(true),
    )
    .arg(
        Arg::with_name("web.systemd-socket")
            .long("web.systemd-socket")
            .help("Use systemd socket activation listeners instead of port listeners (Linux only).")
            .takes_value(false)
            .required(false)
            .default_value(if systemd_socket { "true" } else { "false" }),
    )
    .arg(
        Arg::with_name("web.config.file")
            .long("web.config.file")
            .help("Path to configuration file that can enable TLS or authentication. See: https://github.com/prometheus/exporter-toolkit/blob/master/docs/web-configuration.md")
            .default_value("")
            .takes_value(true),
    )
}

pub fn parse_flags() -> FlagConfig {
    let matches = add_flags(App::new("myapp"), ":9100").get_matches();

    FlagConfig {
        web_listen_addresses: matches
            .values_of("web.listen-address")
            .unwrap()
            .map(String::from)
            .collect(),
        web_systemd_socket: matches.is_present("web.systemd-socket"),
        web_config_file: matches.value_of("web.config.file").unwrap().to_string(),
    }
}