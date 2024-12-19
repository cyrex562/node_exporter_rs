// Copyright 2024 The Prometheus Authors
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

//! Package flag defines standardised flag interactions for use with promslog
//! across Prometheus components.
//! It should typically only ever be imported by main packages.

use clap::{App, Arg};
use promslog::{self, Config, AllowedLevel, AllowedFormat};

// LevelFlagName is the canonical flag name to configure the allowed log level
// within Prometheus projects.
const LEVEL_FLAG_NAME: &str = "log.level";

// LevelFlagHelp is the help description for the log.level flag.
const LEVEL_FLAG_HELP: &str = "Only log messages with the given severity or above. One of: [trace, debug, info, warn, error]";

// FormatFlagName is the canonical flag name to configure the log format
// within Prometheus projects.
const FORMAT_FLAG_NAME: &str = "log.format";

// FormatFlagHelp is the help description for the log.format flag.
const FORMAT_FLAG_HELP: &str = "Output format of log messages. One of: [logfmt, json]";

// AddFlags adds the flags used by this package to the Clap application.
pub fn add_flags(app: &mut App, config: &mut Config) {
    config.level = Some(AllowedLevel::default());
    app.arg(
        Arg::new(LEVEL_FLAG_NAME)
            .long(LEVEL_FLAG_NAME)
            .about(LEVEL_FLAG_HELP)
            .default_value("info")
            .possible_values(&["trace", "debug", "info", "warn", "error"])
            .takes_value(true)
            .value_name("LEVEL")
    );

    config.format = Some(AllowedFormat::default());
    app.arg(
        Arg::new(FORMAT_FLAG_NAME)
            .long(FORMAT_FLAG_NAME)
            .about(FORMAT_FLAG_HELP)
            .default_value("logfmt")
            .possible_values(&["logfmt", "json"])
            .takes_value(true)
            .value_name("FORMAT")
    );
}