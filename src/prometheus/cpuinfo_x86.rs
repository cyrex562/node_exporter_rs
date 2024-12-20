#[cfg(all(target_os = "linux", any(target_arch = "x86", target_arch = "x86_64")))]
use crate::parse_cpu_info_x86 as parse_cpu_info;