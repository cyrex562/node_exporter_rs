// Copyright 2019 The Prometheus Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const IQN_GLOB: &str = "target/iscsi/iqn*";
const TARGET_CORE: &str = "target/core";
const DEVICE_PATH: &str = "devices/rbd";

pub struct FS {
    sysfs: PathBuf,
    configfs: PathBuf,
}

impl FS {
    pub fn new(sysfs_path: &str, configfs_mount_point: &str) -> Result<Self, Box<dyn Error>> {
        let sysfs = if sysfs_path.trim().is_empty() {
            PathBuf::from("/sys")
        } else {
            PathBuf::from(sysfs_path)
        };

        let configfs = if configfs_mount_point.trim().is_empty() {
            PathBuf::from("/sys/kernel/config")
        } else {
            PathBuf::from(configfs_mount_point)
        };

        Ok(FS { sysfs, configfs })
    }

    pub fn path(&self, p: &[&str]) -> PathBuf {
        let mut path = self.configfs.clone();
        for part in p {
            path.push(part);
        }
        path
    }

    pub fn iscsi_stats(&self) -> Result<Vec<Stats>, Box<dyn Error>> {
        let mut stats = Vec::new();
        for entry in glob::glob(&self.path(&[IQN_GLOB]).to_string_lossy())? {
            let path = entry?;
            let s = get_stats(&path)?;
            stats.push(s);
        }
        Ok(stats)
    }
}

#[derive(Debug)]
pub struct TPGT {
    name: String,
    tpgt_path: String,
    is_enable: bool,
    luns: Vec<LUN>,
}

#[derive(Debug)]
pub struct LUN {
    name: String,
    lun_path: String,
    backstore: String,
    object_name: String,
    type_number: String,
}

#[derive(Debug)]
pub struct FILEIO {
    name: String,
    fnumber: String,
    object_name: String,
    filename: String,
}

#[derive(Debug)]
pub struct IBLOCK {
    name: String,
    bnumber: String,
    object_name: String,
    iblock: String,
}

#[derive(Debug)]
pub struct RBD {
    name: String,
    rnumber: String,
    pool: String,
    image: String,
}

#[derive(Debug)]
pub struct RDMCP {
    name: String,
    object_name: String,
}

#[derive(Debug)]
pub struct Stats {
    name: String,
    tpgt: Vec<TPGT>,
    root_path: String,
}

fn get_stats(path: &Path) -> Result<Stats, Box<dyn Error>> {
    // Implementation of get_stats goes here
    Ok(Stats {
        name: path.to_string_lossy().into_owned(),
        tpgt: Vec::new(),
        root_path: path.to_string_lossy().into_owned(),
    })
}

use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub struct Stats {
    name: String,
    root_path: String,
    tpgt: Vec<TPGT>,
}

pub struct TPGT {
    name: String,
    tpgt_path: String,
    is_enable: bool,
    luns: Vec<LUN>,
}

pub struct LUN {
    name: String,
    lun_path: String,
    backstore: String,
    object_name: String,
    type_number: String,
}

pub struct FILEIO {
    name: String,
    fnumber: String,
    object_name: String,
    filename: String,
}

pub struct IBLOCK {
    name: String,
    bnumber: String,
    object_name: String,
    iblock: String,
}

pub struct RBD {
    name: String,
    rnumber: String,
    pool: String,
    image: String,
}

pub struct RDMCP {
    name: String,
    object_name: String,
}

pub fn get_stats(iqn_path: &Path) -> io::Result<Stats> {
    let mut istats = Stats {
        name: iqn_path.file_name().unwrap().to_string_lossy().into_owned(),
        root_path: iqn_path.parent().unwrap().to_string_lossy().into_owned(),
        tpgt: Vec::new(),
    };

    let matches = glob::glob(&format!("{}/tpgt*", iqn_path.display()))?;
    for tpgt_path in matches {
        let tpgt_path = tpgt_path?;
        let mut tpgt = TPGT {
            name: tpgt_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            tpgt_path: tpgt_path.to_string_lossy().into_owned(),
            is_enable: is_path_enable(&tpgt_path)?,
            luns: Vec::new(),
        };

        if tpgt.is_enable {
            let lun_matches = glob::glob(&format!("{}/lun/lun*", tpgt_path.display()))?;
            for lun_path in lun_matches {
                let lun_path = lun_path?;
                if let Ok(lun) = get_lun_link_target(&lun_path) {
                    tpgt.luns.push(lun);
                }
            }
        }
        istats.tpgt.push(tpgt);
    }
    Ok(istats)
}

fn is_path_enable(path: &Path) -> io::Result<bool> {
    let enable_readout = fs::read_to_string(path.join("enable"))?;
    let is_enable = bool::from_str(enable_readout.trim())
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("ParseBool error: {}", e)))?;
    Ok(is_enable)
}

fn get_lun_link_target(lun_path: &Path) -> io::Result<LUN> {
    let mut lun_object = LUN {
        name: lun_path.file_name().unwrap().to_string_lossy().into_owned(),
        lun_path: lun_path.to_string_lossy().into_owned(),
        backstore: String::new(),
        object_name: String::new(),
        type_number: String::new(),
    };

    for entry in fs::read_dir(lun_path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            let target = fs::read_link(entry.path())?;
            let (target_path, object_name) = target.to_string_lossy().rsplit_once('/').unwrap();
            let (_, type_with_number) = target_path.rsplit_once('/').unwrap();

            if let Some(underscore) = type_with_number.rfind('_') {
                lun_object.backstore = type_with_number[..underscore].to_string();
                lun_object.type_number = type_with_number[underscore + 1..].to_string();
            }

            lun_object.object_name = object_name.to_string();
            return Ok(lun_object);
        }
    }
    Err(Error::new(ErrorKind::NotFound, "Lun Link does not exist"))
}

pub fn read_write_ops(iqn_path: &Path, tpgt: &str, lun: &str) -> io::Result<(u64, u64, u64)> {
    let readmb_path = iqn_path
        .join(tpgt)
        .join("lun")
        .join(lun)
        .join("statistics/scsi_tgt_port/read_mbytes");
    let readmb = read_uint_from_file(&readmb_path)?;

    let writemb_path = iqn_path
        .join(tpgt)
        .join("lun")
        .join(lun)
        .join("statistics/scsi_tgt_port/write_mbytes");
    let writemb = read_uint_from_file(&writemb_path)?;

    let iops_path = iqn_path
        .join(tpgt)
        .join("lun")
        .join(lun)
        .join("statistics/scsi_tgt_port/in_cmds");
    let iops = read_uint_from_file(&iops_path)?;

    Ok((readmb, writemb, iops))
}

pub fn get_fileio_udev(fs: &FS, fileio_number: &str, object_name: &str) -> io::Result<FILEIO> {
    let fileio = FILEIO {
        name: format!("fileio_{}", fileio_number),
        fnumber: fileio_number.to_string(),
        object_name: object_name.to_string(),
        filename: String::new(),
    };
    let udev_path = fs.path(&[TARGET_CORE, &fileio.name, &fileio.object_name, "udev_path"]);

    if !udev_path.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("fileio_{} is missing file name", fileio.fnumber),
        ));
    }
    let filename = fs::read_to_string(&udev_path)?;
    Ok(FILEIO {
        filename: filename.trim().to_string(),
        ..fileio
    })
}

pub fn get_iblock_udev(fs: &FS, iblock_number: &str, object_name: &str) -> io::Result<IBLOCK> {
    let iblock = IBLOCK {
        name: format!("iblock_{}", iblock_number),
        bnumber: iblock_number.to_string(),
        object_name: object_name.to_string(),
        iblock: String::new(),
    };
    let udev_path = fs.path(&[TARGET_CORE, &iblock.name, &iblock.object_name, "udev_path"]);

    if !udev_path.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("iblock_{} is missing file name", iblock.bnumber),
        ));
    }
    let filename = fs::read_to_string(&udev_path)?;
    Ok(IBLOCK {
        iblock: filename.trim().to_string(),
        ..iblock
    })
}

pub fn get_rbd_match(fs: &FS, rbd_number: &str, pool_image: &str) -> io::Result<RBD> {
    let rbd = RBD {
        name: format!("rbd_{}", rbd_number),
        rnumber: rbd_number.to_string(),
        pool: String::new(),
        image: String::new(),
    };
    let system_rbds = glob::glob(&fs.path(&[DEVICE_PATH, "[0-9]*"]).to_string_lossy())?;

    for (system_rbd_number, system_rbd_path) in system_rbds.enumerate() {
        let system_rbd_path = system_rbd_path?;
        let system_pool_path = system_rbd_path.join("pool");
        if !system_pool_path.exists() {
            continue;
        }
        let system_pool = fs::read_to_string(&system_pool_path)?.trim().to_string();

        let system_image_path = system_rbd_path.join("name");
        if !system_image_path.exists() {
            continue;
        }
        let system_image = fs::read_to_string(&system_image_path)?.trim().to_string();

        if system_rbd_number.to_string() == rbd_number
            && match_pool_image(&system_pool, &system_image, pool_image)
        {
            return Ok(RBD {
                pool: system_pool,
                image: system_image,
                ..rbd
            });
        }
    }
    Err(Error::new(ErrorKind::NotFound, "RBD match not found"))
}

pub fn get_rdmcp_path(fs: &FS, rdmcp_number: &str, object_name: &str) -> io::Result<RDMCP> {
    let rdmcp = RDMCP {
        name: format!("rd_mcp_{}", rdmcp_number),
        object_name: object_name.to_string(),
    };
    let rdmcp_path = fs.path(&[TARGET_CORE, &rdmcp.name, &rdmcp.object_name]);

    if !rdmcp_path.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("{} does not exist", rdmcp_path.display()),
        ));
    }
    let is_enable = is_path_enable(&rdmcp_path)?;
    if is_enable {
        Ok(rdmcp)
    } else {
        Err(Error::new(ErrorKind::Other, "RDMCP path is not enabled"))
    }
}

fn match_pool_image(pool: &str, image: &str, match_pool_image: &str) -> bool {
    format!("{}-{}", pool, image) == match_pool_image
}

fn read_uint_from_file(path: &Path) -> io::Result<u64> {
    let content = fs::read_to_string(path)?;
    content
        .trim()
        .parse()
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))
}
