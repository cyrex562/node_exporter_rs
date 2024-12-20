use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;

#[derive(Debug)]
struct CPUInfo {
    processor: u32,
    vendor_id: String,
    cpu_family: String,
    model: String,
    model_name: String,
    stepping: String,
    microcode: String,
    cpu_mhz: f64,
    cache_size: String,
    physical_id: String,
    siblings: u32,
    core_id: String,
    cpu_cores: u32,
    apicid: String,
    initial_apicid: String,
    fpu: String,
    fpu_exception: String,
    cpuid_level: u32,
    wp: String,
    flags: Vec<String>,
    bugs: Vec<String>,
    bogomips: f64,
    clflush_size: u32,
    cache_alignment: u32,
    address_sizes: String,
    power_management: String,
}

lazy_static! {
    static ref CPUINFO_CLOCK_REGEX: Regex = Regex::new(r"([\d.]+)").unwrap();
    static ref CPUINFO_S390X_PROCESSOR_REGEX: Regex = Regex::new(r"^processor\s+(\d+):.*").unwrap();
}

fn parse_cpu_info_x86(info: &str) -> Result<Vec<CPUInfo>, Box<dyn std::error::Error>> {
    let mut cpuinfo = Vec::new();
    let mut current_cpu = CPUInfo::default();
    for line in info.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, ": ").collect();
        if parts.len() != 2 {
            continue;
        }
        match parts[0].trim() {
            "processor" => {
                if !current_cpu.processor.is_default() {
                    cpuinfo.push(current_cpu);
                    current_cpu = CPUInfo::default();
                }
                current_cpu.processor = parts[1].parse()?;
            }
            "vendor_id" => current_cpu.vendor_id = parts[1].to_string(),
            "cpu family" => current_cpu.cpu_family = parts[1].to_string(),
            "model" => current_cpu.model = parts[1].to_string(),
            "model name" => current_cpu.model_name = parts[1].to_string(),
            "stepping" => current_cpu.stepping = parts[1].to_string(),
            "microcode" => current_cpu.microcode = parts[1].to_string(),
            "cpu MHz" => current_cpu.cpu_mhz = parts[1].parse()?,
            "cache size" => current_cpu.cache_size = parts[1].to_string(),
            "physical id" => current_cpu.physical_id = parts[1].to_string(),
            "siblings" => current_cpu.siblings = parts[1].parse()?,
            "core id" => current_cpu.core_id = parts[1].to_string(),
            "cpu cores" => current_cpu.cpu_cores = parts[1].parse()?,
            "apicid" => current_cpu.apicid = parts[1].to_string(),
            "initial apicid" => current_cpu.initial_apicid = parts[1].to_string(),
            "fpu" => current_cpu.fpu = parts[1].to_string(),
            "fpu_exception" => current_cpu.fpu_exception = parts[1].to_string(),
            "cpuid level" => current_cpu.cpuid_level = parts[1].parse()?,
            "wp" => current_cpu.wp = parts[1].to_string(),
            "flags" => current_cpu.flags = parts[1].split_whitespace().map(String::from).collect(),
            "bugs" => current_cpu.bugs = parts[1].split_whitespace().map(String::from).collect(),
            "bogomips" => current_cpu.bogomips = parts[1].parse()?,
            "clflush size" => current_cpu.clflush_size = parts[1].parse()?,
            "cache_alignment" => current_cpu.cache_alignment = parts[1].parse()?,
            "address sizes" => current_cpu.address_sizes = parts[1].to_string(),
            "power management" => current_cpu.power_management = parts[1].to_string(),
            _ => {}
        }
    }
    if !current_cpu.processor.is_default() {
        cpuinfo.push(current_cpu);
    }
    Ok(cpuinfo)
}

fn parse_cpu_info_arm(info: &str) -> Result<Vec<CPUInfo>, Box<dyn std::error::Error>> {
    let mut cpuinfo = Vec::new();
    let mut current_cpu = CPUInfo::default();
    let mut features_line = String::new();
    for line in info.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, ": ").collect();
        if parts.len() != 2 {
            continue;
        }
        match parts[0].trim() {
            "processor" => {
                if !current_cpu.processor.is_default() {
                    cpuinfo.push(current_cpu);
                    current_cpu = CPUInfo::default();
                }
                current_cpu.processor = parts[1].parse()?;
            }
            "BogoMIPS" => current_cpu.bogomips = parts[1].parse()?,
            "Features" => features_line = line.to_string(),
            "model name" => current_cpu.model_name = parts[1].to_string(),
            _ => {}
        }
    }
    if !current_cpu.processor.is_default() {
        cpuinfo.push(current_cpu);
    }
    let features: Vec<String> = features_line.split_whitespace().skip(1).map(String::from).collect();
    for cpu in &mut cpuinfo {
        cpu.flags = features.clone();
    }
    Ok(cpuinfo)
}

fn parse_cpu_info_s390x(info: &str) -> Result<Vec<CPUInfo>, Box<dyn std::error::Error>> {
    let mut cpuinfo = Vec::new();
    let mut common_cpu_info = CPUInfo::default();
    for line in info.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, ": ").collect();
        if parts.len() != 2 {
            continue;
        }
        match parts[0].trim() {
            "vendor_id" => common_cpu_info.vendor_id = parts[1].to_string(),
            "bogomips per cpu" => common_cpu_info.bogomips = parts[1].parse()?,
            "features" => common_cpu_info.flags = parts[1].split_whitespace().map(String::from).collect(),
            _ => {}
        }
        if let Some(caps) = CPUINFO_S390X_PROCESSOR_REGEX.captures(line) {
            let mut cpu = common_cpu_info.clone();
            cpu.processor = caps[1].parse()?;
            cpuinfo.push(cpu);
        }
    }
    Ok(cpuinfo)
}

fn parse_cpu_info_mips(info: &str) -> Result<Vec<CPUInfo>, Box<dyn std::error::Error>> {
    let mut cpuinfo = Vec::new();
    let mut current_cpu = CPUInfo::default();
    for line in info.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, ": ").collect();
        if parts.len() != 2 {
            continue;
        }
        match parts[0].trim() {
            "processor" => {
                if !current_cpu.processor.is_default() {
                    cpuinfo.push(current_cpu);
                    current_cpu = CPUInfo::default();
                }
                current_cpu.processor = parts[1].parse()?;
            }
            "cpu model" => current_cpu.model_name = parts[1].to_string(),
            "BogoMIPS" => current_cpu.bogomips = parts[1].parse()?,
            _ => {}
        }
    }
    if !current_cpu.processor.is_default() {
        cpuinfo.push(current_cpu);
    }
    Ok(cpuinfo)
}

fn parse_cpu_info_loong(info: &str) -> Result<Vec<CPUInfo>, Box<dyn std::error::Error>> {
    let mut cpuinfo = Vec::new();
    let mut current_cpu = CPUInfo::default();
    for line in info.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, ": ").collect();
        if parts.len() != 2 {
            continue;
        }
        match parts[0].trim() {
            "processor" => {
                if !current_cpu.processor.is_default() {
                    cpuinfo.push(current_cpu);
                    current_cpu = CPUInfo::default();
                }
                current_cpu.processor = parts[1].parse()?;
            }
            "CPU Family" => current_cpu.cpu_family = parts[1].to_string(),
            "Model Name" => current_cpu.model_name = parts[1].to_string(),
            _ => {}
        }
    }
    if !current_cpu.processor.is_default() {
        cpuinfo.push(current_cpu);
    }
    Ok(cpuinfo)
}

fn parse_cpu_info_ppc(info: &str) -> Result<Vec<CPUInfo>, Box<dyn std::error::Error>> {
    let mut cpuinfo = Vec::new();
    let mut current_cpu = CPUInfo::default();
    for line in info.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, ": ").collect();
        if parts.len() != 2 {
            continue;
        }
        match parts[0].trim() {
            "processor" => {
                if !current_cpu.processor.is_default() {
                    cpuinfo.push(current_cpu);
                    current_cpu = CPUInfo::default();
                }
                current_cpu.processor = parts[1].parse()?;
            }
            "cpu" => current_cpu.vendor_id = parts[1].to_string(),
            "clock" => {
                let clock = CPUINFO_CLOCK_REGEX.find(parts[1]).unwrap().as_str();
                current_cpu.cpu_mhz = clock.parse()?;
            }
            _ => {}
        }
    }
    if !current_cpu.processor.is_default() {
        cpuinfo.push(current_cpu);
    }
    Ok(cpuinfo)
}

fn parse_cpu_info_riscv(info: &str) -> Result<Vec<CPUInfo>, Box<dyn std::error::Error>> {
    let mut cpuinfo = Vec::new();
    let mut current_cpu = CPUInfo::default();
    for line in info.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, ": ").collect();
        if parts.len() != 2 {
            continue;
        }
        match parts[0].trim() {
            "processor" => {
                if !current_cpu.processor.is_default() {
                    cpuinfo.push(current_cpu);
                    current_cpu = CPUInfo::default();
                }
                current_cpu.processor = parts[1].parse()?;
            }
            "hart" => current_cpu.core_id = parts[1].to_string(),
            "isa" => current_cpu.model_name = parts[1].to_string(),
            _ => {}
        }
    }
    if !current_cpu.processor.is_default() {
        cpuinfo.push(current_cpu);
    }
    Ok(cpuinfo)
}

fn parse_cpu_info_dummy(_info: &str) -> Result<Vec<CPUInfo>, Box<dyn std::error::Error>> {
    Err("not implemented".into())
}

fn first_non_empty_line(lines: &mut impl Iterator<Item = &str>) -> Option<&str> {
    lines.find(|line| !line.trim().is_empty())
}