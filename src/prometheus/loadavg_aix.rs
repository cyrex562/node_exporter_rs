use perfstat::CpuTotalStat;
use std::error::Error;

fn get_load() -> Result<Vec<f64>, Box<dyn Error>> {
    let stat = CpuTotalStat::new()?;
    Ok(vec![stat.load_avg_1 as f64, stat.load_avg_5 as f64, stat.load_avg_15 as f64])
}