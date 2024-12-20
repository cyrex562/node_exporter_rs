use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::collections::HashMap;

#[derive(Debug)]
pub struct MountInfo {
    mount_id: i32,
    parent_id: i32,
    major_minor_ver: String,
    root: String,
    mount_point: String,
    options: HashMap<String, String>,
    optional_fields: HashMap<String, String>,
    fs_type: String,
    source: String,
    super_options: HashMap<String, String>,
}

pub fn get_mounts() -> Result<Vec<MountInfo>, io::Error> {
    let data = fs::read_to_string("/proc/self/mountinfo")?;
    parse_mount_info(&data)
}

pub fn get_proc_mounts(pid: i32) -> Result<Vec<MountInfo>, io::Error> {
    let data = fs::read_to_string(format!("/proc/{}/mountinfo", pid))?;
    parse_mount_info(&data)
}

fn parse_mount_info(info: &str) -> Result<Vec<MountInfo>, io::Error> {
    let mut mounts = Vec::new();
    for line in info.lines() {
        let mount = parse_mount_info_string(line)?;
        mounts.push(mount);
    }
    Ok(mounts)
}

fn parse_mount_info_string(mount_string: &str) -> Result<MountInfo, io::Error> {
    let parts: Vec<&str> = mount_string.split_whitespace().collect();
    if parts.len() < 10 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Too few fields in mount string"));
    }

    if parts[parts.len() - 4] != "-" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Couldn't find separator in expected field"));
    }

    let mount_id = parts[0].parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid mount ID"))?;
    let parent_id = parts[1].parse().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid parent ID"))?;

    let mut optional_fields = HashMap::new();
    if parts[6] != "" {
        optional_fields = parse_optional_fields(&parts[6..parts.len() - 4])?;
    }

    Ok(MountInfo {
        mount_id,
        parent_id,
        major_minor_ver: parts[2].to_string(),
        root: parts[3].to_string(),
        mount_point: parts[4].to_string(),
        options: parse_options(parts[5]),
        optional_fields,
        fs_type: parts[parts.len() - 3].to_string(),
        source: parts[parts.len() - 2].to_string(),
        super_options: parse_options(parts[parts.len() - 1]),
    })
}

fn parse_optional_fields(fields: &[&str]) -> Result<HashMap<String, String>, io::Error> {
    let mut optional_fields = HashMap::new();
    for field in fields {
        let parts: Vec<&str> = field.splitn(2, ':').collect();
        let key = parts[0].to_string();
        let value = if parts.len() > 1 { parts[1].to_string() } else { String::new() };
        if is_valid_optional_field(&key) {
            optional_fields.insert(key, value);
        }
    }
    Ok(optional_fields)
}

fn is_valid_optional_field(field: &str) -> bool {
    matches!(field, "shared" | "master" | "propagate_from" | "unbindable")
}

fn parse_options(options: &str) -> HashMap<String, String> {
    let mut opts = HashMap::new();
    for opt in options.split(',') {
        let parts: Vec<&str> = opt.splitn(2, '=').collect();
        let key = parts[0].to_string();
        let value = if parts.len() > 1 { parts[1].to_string() } else { String::new() };
        opts.insert(key, value);
    }
    opts
}