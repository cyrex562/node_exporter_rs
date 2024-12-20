#[cfg(all(
    target_os = "linux",
    not(any(
        target_arch = "x86",
        target_arch = "x86_64",
        target_arch = "arm",
        target_arch = "aarch64",
        target_arch = "loongarch64",
        target_arch = "mips",
        target_arch = "mips64",
        target_arch = "mips64r6",
        target_arch = "mips64el",
        target_arch = "mipsel",
        target_arch = "powerpc64",
        target_arch = "powerpc64le",
        target_arch = "riscv64",
        target_arch = "s390x"
    ))
))]
use crate::parse_cpu_info_dummy as parse_cpu_info;