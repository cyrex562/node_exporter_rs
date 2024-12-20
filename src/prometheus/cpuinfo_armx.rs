#[cfg(all(target_os = "linux", any(target_arch = "arm", target_arch = "aarch64")))]
use crate::parse_cpu_info_arm as parse_cpu_info;