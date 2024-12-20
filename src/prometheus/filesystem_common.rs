use clap::{Arg, Command};
use lazy_static::lazy_static;
use prometheus::{self, core::Collector};
use regex::Regex;
use slog::Logger;
use std::sync::Arc;

lazy_static! {
    static ref MOUNT_POINTS_EXCLUDE: Regex = {
        let matches = Command::new("collector")
            .arg(Arg::new("mount-points-exclude")
                .long("collector.filesystem.mount-points-exclude")
                .about("Regexp of mount points to exclude for filesystem collector. (mutually exclusive to mount-points-include)")
                .default_value(def_mount_points_excluded())
                .takes_value(true))
            .get_matches();
        Regex::new(matches.value_of("mount-points-exclude").unwrap()).unwrap()
    };
}

fn def_mount_points_excluded() -> &'static str {
    "^/(dev|aha)($|/)"
}

struct FilesystemCollector {
    logger: Arc<Logger>,
}

impl FilesystemCollector {
    fn get_stats(&self) -> Result<Vec<FilesystemStats>, Box<dyn std::error::Error>> {
        unimplemented!()
    }
}

struct FilesystemStats {
    labels: FilesystemLabels,
    size: f64,
    free: f64,
    avail: f64,
    files: f64,
    files_free: f64,
    ro: f64,
}

struct FilesystemLabels {
    device: String,
    mount_point: String,
    fs_type: String,
}