use regex::Regex;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::collections::HashMap;

#[derive(Debug)]
pub struct MDStat {
    name: String,
    activity_state: String,
    disks_active: i64,
    disks_total: i64,
    disks_failed: i64,
    disks_down: i64,
    disks_spare: i64,
    blocks_total: i64,
    blocks_synced: i64,
    blocks_to_be_synced: i64,
    blocks_synced_pct: f64,
    blocks_synced_finish_time: f64,
    blocks_synced_speed: f64,
    devices: Vec<String>,
}

pub struct ProcFs {
    proc: String,
}

impl ProcFs {
    pub fn new(proc: &str) -> Self {
        ProcFs { proc: proc.to_string() }
    }

    pub fn md_stat(&self) -> Result<Vec<MDStat>, std::io::Error> {
        let data = fs::read_to_string(Path::new(&self.proc).join("mdstat"))?;
        parse_md_stat(&data)
    }
}

fn parse_md_stat(md_stat_data: &str) -> Result<Vec<MDStat>, std::io::Error> {
    let mut md_stats = Vec::new();
    let lines: Vec<&str> = md_stat_data.lines().collect();

    for (i, &line) in lines.iter().enumerate() {
        if line.trim().is_empty() || line.starts_with(' ') || line.starts_with("Personalities") || line.starts_with("unused") {
            continue;
        }

        let device_fields: Vec<&str> = line.split_whitespace().collect();
        if device_fields.len() < 3 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Expected 3+ fields, got {}", line)));
        }

        let md_name = device_fields[0].to_string();
        let state = device_fields[2].to_string();

        if lines.len() <= i + 3 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Too few lines for md device: {}", md_name)));
        }

        let fail = line.matches("(F)").count() as i64;
        let spare = line.matches("(S)").count() as i64;
        let (active, total, down, size) = eval_status_line(lines[i], lines[i + 1])?;

        let mut sync_line_idx = i + 2;
        if lines[i + 2].contains("bitmap") {
            sync_line_idx += 1;
        }

        let mut blocks_synced = size;
        let mut blocks_to_be_synced = size;
        let mut speed = 0.0;
        let mut finish = 0.0;
        let mut pct = 0.0;
        let recovering = lines[sync_line_idx].contains("recovery");
        let resyncing = lines[sync_line_idx].contains("resync");
        let checking = lines[sync_line_idx].contains("check");

        let state = if recovering {
            "recovering".to_string()
        } else if checking {
            "checking".to_string()
        } else if resyncing {
            "resyncing".to_string()
        } else {
            state
        };

        if recovering || resyncing || checking {
            if lines[sync_line_idx].contains("PENDING") || lines[sync_line_idx].contains("DELAYED") {
                blocks_synced = 0;
            } else {
                let (bs, bts, p, f, s) = eval_recovery_line(lines[sync_line_idx])?;
                blocks_synced = bs;
                blocks_to_be_synced = bts;
                pct = p;
                finish = f;
                speed = s;
            }
        }

        md_stats.push(MDStat {
            name: md_name,
            activity_state: state,
            disks_active: active,
            disks_failed: fail,
            disks_down: down,
            disks_spare: spare,
            disks_total: total,
            blocks_total: size,
            blocks_synced,
            blocks_to_be_synced,
            blocks_synced_pct: pct,
            blocks_synced_finish_time: finish,
            blocks_synced_speed: speed,
            devices: eval_component_devices(&device_fields),
        });
    }

    Ok(md_stats)
}

fn eval_status_line(device_line: &str, status_line: &str) -> Result<(i64, i64, i64, i64), std::io::Error> {
    let status_fields: Vec<&str> = status_line.split_whitespace().collect();
    if status_fields.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unexpected statusline {}", status_line)));
    }

    let size = i64::from_str(status_fields[0])?;
    if device_line.contains("raid0") || device_line.contains("linear") {
        let total = device_line.matches('[').count() as i64;
        return Ok((total, total, 0, size));
    }

    if device_line.contains("inactive") {
        return Ok((0, 0, 0, size));
    }

    let status_line_re = Regex::new(r"(\d+) blocks .*\[(\d+)/(\d+)\] \[([U_]+)\]").unwrap();
    let matches = status_line_re.captures(status_line).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Could not find all substring matches {}", status_line)))?;

    let total = i64::from_str(&matches[2])?;
    let active = i64::from_str(&matches[3])?;
    let down = matches[4].matches('_').count() as i64;

    Ok((active, total, down, size))
}

fn eval_recovery_line(recovery_line: &str) -> Result<(i64, i64, f64, f64, f64), std::io::Error> {
    let recovery_line_blocks_re = Regex::new(r"\((\d+/\d+)\)").unwrap();
    let recovery_line_pct_re = Regex::new(r"= (.+)%").unwrap();
    let recovery_line_finish_re = Regex::new(r"finish=(.+)min").unwrap();
    let recovery_line_speed_re = Regex::new(r"speed=(.+)[A-Z]").unwrap();

    let matches = recovery_line_blocks_re.captures(recovery_line).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unexpected recoveryLine blocks {}", recovery_line)))?;
    let blocks: Vec<&str> = matches[1].split('/').collect();
    let blocks_synced = i64::from_str(blocks[0])?;
    let blocks_to_be_synced = i64::from_str(blocks[1])?;

    let matches = recovery_line_pct_re.captures(recovery_line).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unexpected recoveryLine matching percentage {}", recovery_line)))?;
    let pct = f64::from_str(matches[1].trim())?;

    let matches = recovery_line_finish_re.captures(recovery_line).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unexpected recoveryLine matching est. finish time: {}", recovery_line)))?;
    let finish = f64::from_str(matches[1].trim())?;

    let matches = recovery_line_speed_re.captures(recovery_line).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unexpected recoveryLine value: {}", recovery_line)))?;
    let speed = f64::from_str(matches[1].trim())?;

    Ok((blocks_synced, blocks_to_be_synced, pct, finish, speed))
}

fn eval_component_devices(device_fields: &[&str]) -> Vec<String> {
    let component_device_re = Regex::new(r"(.*)\[\d+\]").unwrap();
    device_fields.iter().skip(4).filter_map(|&field| {
        component_device_re.captures(field).map(|cap| cap[1].to_string())
    }).collect()
}