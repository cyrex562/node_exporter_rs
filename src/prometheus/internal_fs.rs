use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_PROC_MOUNT_POINT: &str = "/proc";
const DEFAULT_SYS_MOUNT_POINT: &str = "/sys";
const DEFAULT_CONFIGFS_MOUNT_POINT: &str = "/sys/kernel/config";
const DEFAULT_SELINUX_MOUNT_POINT: &str = "/sys/fs/selinux";

#[derive(Debug)]
pub struct FsError {
    details: String,
}

impl FsError {
    fn new(msg: &str) -> FsError {
        FsError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for FsError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub struct FS {
    mount_point: PathBuf,
}

impl FS {
    pub fn new(mount_point: &str) -> Result<FS, Box<dyn std::error::Error>> {
        let path = Path::new(mount_point);
        let metadata = fs::metadata(path)?;

        if !metadata.is_dir() {
            return Err(Box::new(FsError::new(&format!(
                "mount point {} is not a directory",
                mount_point
            ))));
        }

        Ok(FS {
            mount_point: path.to_path_buf(),
        })
    }

    pub fn path(&self, p: &[&str]) -> PathBuf {
        let mut full_path = self.mount_point.clone();
        for part in p {
            full_path.push(part);
        }
        full_path
    }
}

// fn main() {
//     // Example usage
//     match FS::new(DEFAULT_PROC_MOUNT_POINT) {
//         Ok(fs) => {
//             let path = fs.path(&["some", "path"]);
//             println!("Full path: {:?}", path);
//         }
//         Err(e) => println!("Error: {}", e),
//     }
// }