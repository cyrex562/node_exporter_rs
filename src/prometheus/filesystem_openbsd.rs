use libc::{self, statfs, getfsstat, MNT_NOWAIT, MNT_RDONLY};
use std::ffi::CStr;
use std::ptr;
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
        let mut mnt: Vec<statfs> = Vec::new();
        let size = unsafe { getfsstat(ptr::null_mut(), 0, MNT_NOWAIT) };
        if size == -1 {
            return Err("getfsstat() failed".into());
        }
        mnt.resize(size as usize, unsafe { std::mem::zeroed() });

        let size = unsafe { getfsstat(mnt.as_mut_ptr(), (mnt.len() * std::mem::size_of::<statfs>()) as i64, MNT_NOWAIT) };
        if size == -1 {
            return Err("getfsstat() failed".into());
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

            let ro = if v.f_flags & MNT_RDONLY != 0 { 1.0 } else { 0.0 };

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