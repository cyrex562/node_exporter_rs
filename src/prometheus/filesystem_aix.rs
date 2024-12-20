use perfstat::FileSystemStat;
use slog::Logger;

const DEF_MOUNT_POINTS_EXCLUDED: &str = "^/(dev|aha)($|/)";
const DEF_FS_TYPES_EXCLUDED: &str = "^procfs$";

struct FilesystemCollector {
    mount_point_filter: Filter,
    fs_type_filter: Filter,
    logger: Logger,
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
        let fs_stat = FileSystemStat::collect()?;
        let mut stats = Vec::new();

        for stat in fs_stat {
            if self.mount_point_filter.ignored(&stat.mount_point) {
                self.logger.debug("Ignoring mount point", o!("mountpoint" => stat.mount_point.clone()));
                continue;
            }
            let fstype = stat.type_string();
            if self.fs_type_filter.ignored(&fstype) {
                self.logger.debug("Ignoring fs type", o!("type" => fstype.clone()));
                continue;
            }

            let ro = if stat.flags & perfstat::VFS_READONLY != 0 { 1.0 } else { 0.0 };

            stats.push(FilesystemStats {
                labels: FilesystemLabels {
                    device: stat.device.clone(),
                    mount_point: stat.mount_point.clone(),
                    fs_type: fstype,
                },
                size: stat.total_blocks as f64 / 512.0,
                free: stat.free_blocks as f64 / 512.0,
                avail: stat.free_blocks as f64 / 512.0,
                files: stat.total_inodes as f64,
                files_free: stat.free_inodes as f64,
                ro,
            });
        }
        Ok(stats)
    }
}