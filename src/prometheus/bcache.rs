use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use glob::glob;

pub struct Stats {
    // The name of the bcache used to source these statistics.
    pub name: String,
    pub bcache: BcacheStats,
    pub bdevs: Vec<BdevStats>,
    pub caches: Vec<CacheStats>,
}

// BcacheStats contains statistics tied to a bcache ID.
pub struct BcacheStats {
    pub average_key_size: u64,
    pub btree_cache_size: u64,
    pub cache_available_percent: u64,
    pub congested: u64,
    pub root_usage_percent: u64,
}

// BdevStats contains statistics for a bcache backing device.
pub struct BdevStats {
    pub name: String,
    pub dirty_data: u64,
    pub io_errors: u64,
    pub metadata_written: u64,
    pub written: u64,
}

// CacheStats contains statistics for a bcache cache device.
pub struct CacheStats {
    pub name: String,
    pub io_errors: u64,
    pub metadata_written: u64,
    pub written: u64,
}



pub struct FS {
    sys: Arc<Mutex<PathBuf>>,
}

impl FS {
    pub fn new_default_fs() -> io::Result<Self> {
        Self::new(fs::default_sys_mount_point())
    }

    pub fn new(mount_point: String) -> io::Result<Self> {
        let mount_point = if mount_point.trim().is_empty() {
            fs::default_sys_mount_point()
        } else {
            PathBuf::from(mount_point)
        };

        if mount_point.exists() {
            Ok(FS {
                sys: Arc::new(Mutex::new(mount_point)),
            })
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Mount point can't be read",
            ))
        }
    }

    pub fn stats(&self) -> io::Result<Vec<Stats>> {
        self.stats_internal(true)
    }

    pub fn stats_without_priority(&self) -> io::Result<Vec<Stats>> {
        self.stats_internal(false)
    }

    fn stats_internal(&self, priority_stats: bool) -> io::Result<Vec<Stats>> {
        let sys_path = self.sys.lock().unwrap();
        let pattern = sys_path.join("fs/bcache/*-*").to_string_lossy().to_string();
        let matches = glob(&pattern).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut stats = Vec::new();
        for entry in matches {
            match entry {
                Ok(uuid_path) => {
                    let name = uuid_path.file_name().unwrap().to_string_lossy().to_string();
                    match get_stats(&uuid_path, priority_stats) {
                        Ok(s) => stats.push(s),
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
            }
        }

        Ok(stats)
    }
}

mod fs {
    use std::path::PathBuf;

    pub fn default_sys_mount_point() -> PathBuf {
        PathBuf::from("/sys")
    }
}

pub struct Stats {
    // Define the fields for Stats struct
}

fn get_stats(path: &Path, priority_stats: bool) -> io::Result<Stats> {
    // Implement the logic to get stats from the given path
    Ok(Stats {
        // Initialize the fields
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_fs_bcache_stats() {
        let bcache = FS::new("testdata/fixtures/sys".to_string()).expect("failed to access bcache fs");
        let stats = bcache.stats().expect("failed to parse bcache stats");

        let tests = vec![
            TestStats {
                name: "deaddd54-c735-46d5-868e-f331c5fd7c74".to_string(),
                bdevs: 1,
                caches: 1,
            },
        ];

        for test in tests {
            let stat = stats.iter().find(|s| s.name == test.name).expect("stat not found");
            assert_eq!(stat.bdevs.len(), test.bdevs);
            assert_eq!(stat.caches.len(), test.caches);
        }
    }

    struct TestStats {
        name: String,
        bdevs: usize,
        caches: usize,
    }
}