use libc::{self, c_char, c_uint, c_ulong, c_void};
use slog::Logger;
use std::ffi::CStr;
use std::ptr;
use std::sync::Arc;

const DEF_MOUNT_POINTS_EXCLUDED: &str = "^/(dev)($|/)";
const DEF_FS_TYPES_EXCLUDED: &str = "^(kernfs|procfs|ptyfs|fdesc)$";
const VFS_NAMELEN: usize = 32;
const VFS_MNAMELEN: usize = 1024;

#[repr(C)]
struct Statvfs90 {
    f_flag: c_uint,
    f_bsize: c_uint,
    f_frsize: c_uint,
    f_iosize: c_uint,
    f_blocks: u64,
    f_bfree: u64,
    f_bavail: u64,
    f_bresvd: u64,
    f_files: u64,
    f_ffree: u64,
    f_favail: u64,
    f_fresvd: u64,
    f_syncreads: u64,
    f_syncwrites: u64,
    f_asyncreads: u64,
    f_asyncwrites: u64,
    f_fsidx: [u32; 2],
    f_fsid: u32,
    f_namemax: c_uint,
    f_owner: u32,
    f_spare: [u32; 4],
    f_fstypename: [c_char; VFS_NAMELEN],
    f_mntonname: [c_char; VFS_MNAMELEN],
    f_mntfromname: [c_char; VFS_MNAMELEN],
    cgo_pad: [u8; 4],
}

struct FilesystemCollector {
    mount_point_filter: Filter,
    fs_type_filter: Filter,
    logger: Arc<Logger>,
}

struct FilesystemStats {
    labels: FilesystemLabels,
    size: f64,
    free: f64,
    avail: f64,
    files: f64,
    files_free: f64,
    ro: f64,
}

struct FilesystemLabels {
    device: String,
    mount_point: String,
    fs_type: String,
}

impl FilesystemCollector {
    fn get_stats(&self) -> Result<Vec<FilesystemStats>, Box<dyn std::error::Error>> {
        let mut mnt: Vec<Statvfs90> = Vec::new();
        let mut buf_size = 0;

        loop {
            let r1 = unsafe { libc::syscall(libc::SYS_getvfsstat, ptr::null_mut::<c_void>(), 0, libc::ST_NOWAIT) };
            if r1 == -1 {
                return Err("getvfsstat: ABI mismatch".into());
            }
            buf_size = r1 as usize;
            mnt.resize(buf_size, unsafe { std::mem::zeroed() });

            let r2 = unsafe {
                libc::syscall(
                    libc::SYS_getvfsstat,
                    mnt.as_mut_ptr() as *mut c_void,
                    (mnt.len() * std::mem::size_of::<Statvfs90>()) as c_ulong,
                    libc::ST_NOWAIT,
                )
            };
            if r1 == r2 {
                break;
            }
        }

        let mut stats = Vec::new();
        for v in mnt {
            let mount_point = unsafe { CStr::from_ptr(v.f_mntonname.as_ptr()) }.to_string_lossy().into_owned();
            if self.mount_point_filter.ignored(&mount_point) {
                self.logger.debug("Ignoring mount point", o!("mountpoint" => mount_point.clone()));
                continue;
            }

            let device = unsafe { CStr::from_ptr(v.f_mntfromname.as_ptr()) }.to_string_lossy().into_owned();
            let fs_type = unsafe { CStr::from_ptr(v.f_fstypename.as_ptr()) }.to_string_lossy().into_owned();
            if self.fs_type_filter.ignored(&fs_type) {
                self.logger.debug("Ignoring fs type", o!("type" => fs_type.clone()));
                continue;
            }

            let ro = if v.f_flag & libc::MNT_RDONLY != 0 { 1.0 } else { 0.0 };

            stats.push(FilesystemStats {
                labels: FilesystemLabels {
                    device,
                    mount_point,
                    fs_type,
                },
                size: v.f_blocks as f64 * v.f_bsize as f64,
                free: v.f_bfree as f64 * v.f_bsize as f64,
                avail: v.f_bavail as f64 * v.f_bsize as f64,
                files: v.f_files as f64,
                files_free: v.f_ffree as f64,
                ro,
            });
        }
        Ok(stats)
    }
}