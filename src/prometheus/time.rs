use std::error::Error;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::num::ParseFloatError;

#[derive(Debug)]
struct NaNOrInfError;

impl fmt::Display for NaNOrInfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value is NaN or Inf")
    }
}

impl Error for NaNOrInfError {}

fn convert_to_float(value: &dyn std::any::Any) -> Result<f64, Box<dyn Error>> {
    if let Some(v) = value.downcast_ref::<f64>() {
        Ok(*v)
    } else if let Some(v) = value.downcast_ref::<String>() {
        v.parse::<f64>().map_err(|e| Box::new(e) as Box<dyn Error>)
    } else if let Some(v) = value.downcast_ref::<i32>() {
        Ok(*v as f64)
    } else if let Some(v) = value.downcast_ref::<u32>() {
        Ok(*v as f64)
    } else if let Some(v) = value.downcast_ref::<i64>() {
        Ok(*v as f64)
    } else if let Some(v) = value.downcast_ref::<u64>() {
        Ok(*v as f64)
    } else if let Some(v) = value.downcast_ref::<Duration>() {
        Ok(v.as_secs_f64())
    } else {
        Err(Box::new(fmt::Error) as Box<dyn Error>)
    }
}

fn float_to_time(value: f64) -> Result<SystemTime, Box<dyn Error>> {
    if value.is_nan() || value.is_infinite() {
        return Err(Box::new(NaNOrInfError));
    }
    let timestamp = value * 1e9;
    if timestamp > i64::MAX as f64 || timestamp < i64::MIN as f64 {
        return Err(Box::new(fmt::Error) as Box<dyn Error>);
    }
    let duration = Duration::from_nanos(timestamp as u64);
    Ok(UNIX_EPOCH + duration)
}

fn humanize_duration(value: &dyn std::any::Any) -> Result<String, Box<dyn Error>> {
    let v = convert_to_float(value)?;
    if v.is_nan() || v.is_infinite() {
        return Ok(format!("{:.4}", v));
    }
    if v == 0.0 {
        return Ok(format!("{:.4}s", v));
    }
    if v.abs() >= 1.0 {
        let sign = if v < 0.0 { "-" } else { "" };
        let mut v = v.abs();
        let duration = v as i64;
        let seconds = duration % 60;
        let minutes = (duration / 60) % 60;
        let hours = (duration / 60 / 60) % 24;
        let days = duration / 60 / 60 / 24;
        if days != 0 {
            return Ok(format!("{}{}d {}h {}m {}s", sign, days, hours, minutes, seconds));
        }
        if hours != 0 {
            return Ok(format!("{}{}h {}m {}s", sign, hours, minutes, seconds));
        }
        if minutes != 0 {
            return Ok(format!("{}{}m {}s", sign, minutes, seconds));
        }
        return Ok(format!("{}{}.4gs", sign, v));
    }
    let mut prefix = "";
    let mut v = v;
    for p in &["m", "u", "n", "p", "f", "a", "z", "y"] {
        if v.abs() >= 1.0 {
            break;
        }
        prefix = p;
        v *= 1000.0;
    }
    Ok(format!("{:.4}{}s", v, prefix))
}

fn humanize_timestamp(value: &dyn std::any::Any) -> Result<String, Box<dyn Error>> {
    let v = convert_to_float(value)?;
    match float_to_time(v) {
        Ok(tm) => Ok(format!("{:?}", tm)),
        Err(e) if e.downcast_ref::<NaNOrInfError>().is_some() => Ok(format!("{:.4}", v)),
        Err(e) => Err(e),
    }
}

// fn main() {
//     // Example usage
//     let duration = Duration::new(12345, 0);
//     let result = humanize_duration(&duration);
//     println!("{:?}", result);

//     let timestamp = 1633072800.0;
//     let result = humanize_timestamp(&timestamp);
//     println!("{:?}", result);
// }