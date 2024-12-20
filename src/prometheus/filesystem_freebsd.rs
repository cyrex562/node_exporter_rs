use libc::{self, statfs, getfsstat, MNT_NOWAIT, MNT_IGNORE, MNT_RDONLY};
use std::ffi::CStr;
use std::ptr;
use std::slice;
use std::sync::Arc;

const DEF_MOUNT_POINTS_EXCLUDED: &str = "^/(dev)($|/)";
const DEF_FS_TYPES_EXCLUDED: &str = "^devfs$";

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
        let mut buf: Vec<statfs> = Vec::with_capacity(256);
        let n = unsafe { getfsstat(buf.as_mut_ptr(), buf.capacity() as i64, MNT_NOWAIT) };
        if n == -1 {
            return Err("getfsstat() failed".into());
        }
        unsafe { buf.set_len(n as usize) };

        let mut stats = Vec::new();
        for fs in buf {
            let mount_point = unsafe { CStr::from_ptr(fs.f_mntonname.as_ptr()) }.to_string_lossy().into_owned();
            if self.mount_point_filter.ignored(&mount_point) {
                self.logger.debug("Ignoring mount point", o!("mountpoint" => mount_point.clone()));
                continue;
            }

            let device = unsafe { CStr::from_ptr(fs.f_mntfromname.as_ptr()) }.to_string_lossy().into_owned();
            let fs_type = unsafe { CStr::from_ptr(fs.f_fstypename.as_ptr()) }.to_string_lossy().into_owned();
            if self.fs_type_filter.ignored(&fs_type) {
                self.logger.debug("Ignoring fs type", o!("type" => fs_type.clone()));
                continue;
            }

            if fs.f_flags & MNT_IGNORE != 0 {
                self.logger.debug("Ignoring mount flagged as ignore", o!("mountpoint" => mount_point.clone()));
                continue;
            }

            let ro = if fs.f_flags & MNT_RDONLY != 0 { 1.0 } else { 0.0 };

            stats.push(FilesystemStats {
                labels: FilesystemLabels {
                    device,
                    mount_point: rootfs_strip_prefix(&mount_point),
                    fs_type,
                },
                size: fs.f_blocks as f64 * fs.f_bsize as f64,
                free: fs.f_bfree as f64 * fs.f_bsize as f64,
                avail: fs.f_bavail as f64 * fs.f_bsize as f64,
                files: fs.f_files as f64,
                files_free: fs.f_ffree as f64,
                ro,
            });
        }
        Ok(stats)
    }
}