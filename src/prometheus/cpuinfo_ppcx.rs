#[cfg(all(target_os = "linux", any(target_arch = "powerpc64", target_arch = "powerpc64le")))]
use crate::parse_cpu_info_ppc as parse_cpu_info;