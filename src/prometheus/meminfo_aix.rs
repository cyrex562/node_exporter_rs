use slog::Logger;
use perfstat::MemoryTotalStat;
use std::collections::HashMap;
use std::error::Error;

struct MeminfoCollector {
    logger: Logger,
}

impl MeminfoCollector {
    fn new(logger: Logger) -> Result<Self, Box<dyn Error>> {
        Ok(MeminfoCollector { logger })
    }

    fn get_mem_info(&self) -> Result<HashMap<String, f64>, Box<dyn Error>> {
        let stats = MemoryTotalStat::new()?;
        let mut mem_info = HashMap::new();
        mem_info.insert("total_bytes".to_string(), (stats.real_total * 4096) as f64);
        mem_info.insert("free_bytes".to_string(), (stats.real_free * 4096) as f64);
        mem_info.insert("available_bytes".to_string(), (stats.real_available * 4096) as f64);
        Ok(mem_info)
    }
}