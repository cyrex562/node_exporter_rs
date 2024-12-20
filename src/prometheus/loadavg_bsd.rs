use std::error::Error;
use std::mem;
use nix::sys::sysctl::sysctl;

#[repr(C)]
struct Loadavg {
    load: [u32; 3],
    scale: i32,
}

fn get_load() -> Result<Vec<f64>, Box<dyn Error>> {
    let load: Loadavg = unsafe { mem::transmute(sysctl::<[u8; mem::size_of::<Loadavg>()]>("vm.loadavg")?) };
    let scale = load.scale as f64;
    Ok(vec![
        load.load[0] as f64 / scale,
        load.load[1] as f64 / scale,
        load.load[2] as f64 / scale,
    ])
}