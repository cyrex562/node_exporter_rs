use prometheus::{self, core::{Collector, Desc, Metric, Opts, ValueType}};
use slog::Logger;
use std::error::Error;

struct LoadavgCollector {
    metrics: Vec<TypedDesc>,
    logger: Logger,
}

impl LoadavgCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn Error>> {
        let metrics = vec![
            TypedDesc {
                desc: Desc::new("node_load1", "1m load average.", vec![], std::collections::HashMap::new()),
                value_type: ValueType::Gauge,
            },
            TypedDesc {
                desc: Desc::new("node_load5", "5m load average.", vec![], std::collections::HashMap::new()),
                value_type: ValueType::Gauge,
            },
            TypedDesc {
                desc: Desc::new("node_load15", "15m load average.", vec![], std::collections::HashMap::new()),
                value_type: ValueType::Gauge,
            },
        ];

        Ok(LoadavgCollector { metrics, logger })
    }

    fn update(&self, ch: &mut dyn FnMut(Box<dyn Metric>)) -> Result<(), Box<dyn Error>> {
        let loads = get_load()?;
        for (i, &load) in loads.iter().enumerate() {
            self.logger.debug("return load", &["index", &i.to_string(), "load", &load.to_string()]);
            ch(Box::new(prometheus::Gauge::new(self.metrics[i].desc.clone(), load, vec![])));
        }
        Ok(())
    }
}

fn get_load() -> Result<Vec<f64>, Box<dyn Error>> {
    // Implementation depends on the platform-specific code
    Ok(vec![0.0, 0.0, 0.0]) // Placeholder
}

struct TypedDesc {
    desc: Desc,
    value_type: ValueType,
}

use std::fs;
use std::io::Error;
use std::num::ParseFloatError;
use std::path::Path;

#[derive(Debug)]
pub struct LoadAvg {
    load1: f64,
    load5: f64,
    load15: f64,
}

pub struct ProcFs {
    proc: String,
}

impl ProcFs {
    pub fn new(proc: &str) -> Self {
        ProcFs { proc: proc.to_string() }
    }

    pub fn load_avg(&self) -> Result<LoadAvg, Error> {
        let path = Path::new(&self.proc).join("loadavg");
        let data = fs::read_to_string(path)?;
        parse_load(&data)
    }
}

fn parse_load(data: &str) -> Result<LoadAvg, ParseFloatError> {
    let parts: Vec<&str> = data.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(ParseFloatError::from(std::num::IntErrorKind::Empty));
    }
    let load1 = parts[0].parse()?;
    let load5 = parts[1].parse()?;
    let load15 = parts[2].parse()?;
    Ok(LoadAvg { load1, load5, load15 })
}