use regex::Regex;
use std::fs;
use std::num::ParseIntError;
use std::path::Path;

fn read_uint_from_file(path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(path)?;
    let value = data.trim().parse::<u64>()?;
    Ok(value)
}

lazy_static! {
    static ref METRIC_NAME_REGEX: Regex = Regex::new(r"[^0-9A-Za-z_]+").unwrap();
}

fn sanitize_metric_name(metric_name: &str) -> String {
    METRIC_NAME_REGEX.replace_all(metric_name, "_").to_string()
}