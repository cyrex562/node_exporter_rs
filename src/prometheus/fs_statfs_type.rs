use libc::{self, statfs};
use std::ffi::CString;
use std::ptr;

const PROC_SUPER_MAGIC: i64 = 0x9fa0;

// is_real_proc determines whether supplied mountpoint is really a proc filesystem.
fn is_real_proc(mount_point: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let c_mount_point = CString::new(mount_point)?;
    let mut stat: statfs = unsafe { std::mem::zeroed() };
    let res = unsafe { libc::statfs(c_mount_point.as_ptr(), &mut stat) };
    if res != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(stat.f_type == PROC_SUPER_MAGIC)
}