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

//! Package promslog defines standardised ways to initialize the Rust standard
//! library's log logger.
//! It should typically only ever be imported by main packages.

use std::fmt;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, PartialEq)]
pub enum LogStyle {
    SlogStyle,
    GoKitStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowedLevel {
    s: String,
    lvl: LevelFilter,
}

impl AllowedLevel {
    pub fn set(&mut self, s: &str) -> Result<(), String> {
        self.s = s.to_string();
        self.lvl = match s.to_lowercase().as_str() {
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            _ => return Err(format!("unrecognized log level {}", s)),
        };
        Ok(())
    }
}

impl fmt::Display for AllowedLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.s)
    }
}

impl<'de> Deserialize<'de> for AllowedLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut level = AllowedLevel {
            s: s.clone(),
            lvl: LevelFilter::Off,
        };
        level.set(&s).map_err(serde::de::Error::custom)?;
        Ok(level)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowedFormat {
    s: String,
}

impl AllowedFormat {
    pub fn set(&mut self, s: &str) -> Result<(), String> {
        match s {
            "logfmt" | "json" => {
                self.s = s.to_string();
                Ok(())
            }
            _ => Err(format!("unrecognized log format {}", s)),
        }
    }
}

impl fmt::Display for AllowedFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.s)
    }
}

impl<'de> Deserialize<'de> for AllowedFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut format = AllowedFormat { s: s.clone() };
        format.set(&s).map_err(serde::de::Error::custom)?;
        Ok(format)
    }
}

pub struct Config {
    pub level: Option<AllowedLevel>,
    pub format: Option<AllowedFormat>,
    pub style: LogStyle,
    pub writer: Option<Box<dyn Write + Send>>,
}

pub struct Logger {
    level: LevelFilter,
    writer: Mutex<Box<dyn Write + Send>>,
}

impl Logger {
    pub fn new(config: &Config) -> Self {
        let level = config.level.as_ref().map_or(LevelFilter::Info, |l| l.lvl);
        let writer = config.writer.clone().unwrap_or_else(|| Box::new(io::stderr()));
        Logger {
            level,
            writer: Mutex::new(writer),
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut writer = self.writer.lock().unwrap();
            writeln!(writer, "{} - {}", record.level(), record.args()).unwrap();
        }
    }

    fn flush(&self) {}
}

pub fn init_logger(config: &Config) -> Result<(), SetLoggerError> {
    let logger = Logger::new(config);
    log::set_max_level(logger.level);
    log::set_boxed_logger(Box::new(logger))
}

pub fn new_nop_logger() -> Logger {
    Logger {
        level: LevelFilter::Off,
        writer: Mutex::new(Box::new(io::sink())),
    }
}