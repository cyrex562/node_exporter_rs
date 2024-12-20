#[cfg(all(target_os = "linux", any(target_arch = "riscv32", target_arch = "riscv64")))]
use crate::parse_cpu_info_riscv as parse_cpu_info;