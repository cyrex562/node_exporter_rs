use libc::{self, statfs, getmntinfo, MNT_NOWAIT};
use std::ffi::CStr;
use std::ptr;
use std::slice;
use std::sync::Arc;

const DEF_MOUNT_POINTS_EXCLUDED: &str = "^/(dev)($|/)";
const DEF_FS_TYPES_EXCLUDED: &str = "^devfs$";
const READ_ONLY: u32 = 0x1; // MNT_RDONLY

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
        let mut mntbuf: *mut statfs = ptr::null_mut();
        let count = unsafe { getmntinfo(&mut mntbuf, MNT_NOWAIT) };
        if count == 0 {
            return Err("getmntinfo() failed".into());
        }

        let mnt = unsafe { slice::from_raw_parts(mntbuf, count as usize) };
        let mut stats = Vec::new();

        for mnt in mnt {
            let mount_point = unsafe { CStr::from_ptr(mnt.f_mntonname.as_ptr()) }.to_string_lossy().into_owned();
            if self.mount_point_filter.ignored(&mount_point) {
                self.logger.debug("Ignoring mount point", o!("mountpoint" => mount_point.clone()));
                continue;
            }

            let device = unsafe { CStr::from_ptr(mnt.f_mntfromname.as_ptr()) }.to_string_lossy().into_owned();
            let fs_type = unsafe { CStr::from_ptr(mnt.f_fstypename.as_ptr()) }.to_string_lossy().into_owned();
            if self.fs_type_filter.ignored(&fs_type) {
                self.logger.debug("Ignoring fs type", o!("type" => fs_type.clone()));
                continue;
            }

            let ro = if mnt.f_flags & READ_ONLY != 0 { 1.0 } else { 0.0 };

            stats.push(FilesystemStats {
                labels: FilesystemLabels {
                    device,
                    mount_point: rootfs_strip_prefix(&mount_point),
                    fs_type,
                },
                size: mnt.f_blocks as f64 * mnt.f_bsize as f64,
                free: mnt.f_bfree as f64 * mnt.f_bsize as f64,
                avail: mnt.f_bavail as f64 * mnt.f_bsize as f64,
                files: mnt.f_files as f64,
                files_free: mnt.f_ffree as f64,
                ro,
            });
        }
        Ok(stats)
    }
}