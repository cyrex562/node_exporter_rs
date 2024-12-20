use std::fs;
use std::io::Error;
use std::num::ParseFloatError;

fn get_load() -> Result<Vec<f64>, Error> {
    let data = fs::read_to_string("/proc/loadavg")?;
    parse_load(&data)
}

fn parse_load(data: &str) -> Result<Vec<f64>, ParseFloatError> {
    let parts: Vec<&str> = data.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(ParseFloatError::from(std::num::IntErrorKind::Empty));
    }
    let mut loads = Vec::with_capacity(3);
    for load in &parts[0..3] {
        loads.push(load.parse()?);
    }
    Ok(loads)
}