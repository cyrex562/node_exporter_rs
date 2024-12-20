use std::error::Error;
use std::ffi::CString;
use std::os::raw::c_double;
use kstat::{Kstat, KstatCtl};

const FSCALE: c_double = 1 << 8;

fn kstat_to_float(ks: &Kstat, kstat_key: &str) -> Result<f64, Box<dyn Error>> {
    let kstat_value = ks.get_named(kstat_key)?;
    let kstat_loadavg = (kstat_value.uint_val() as f64) / FSCALE;
    Ok(kstat_loadavg)
}

fn get_load() -> Result<Vec<f64>, Box<dyn Error>> {
    let ctl = KstatCtl::new()?;
    let ks = ctl.lookup("unix", 0, "system_misc")?;

    let loadavg1_min = kstat_to_float(&ks, "avenrun_1min")?;
    let loadavg5_min = kstat_to_float(&ks, "avenrun_5min")?;
    let loadavg15_min = kstat_to_float(&ks, "avenrun_15min")?;

    Ok(vec![loadavg1_min, loadavg5_min, loadavg15_min])
}