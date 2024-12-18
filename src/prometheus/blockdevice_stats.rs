use std::fs::{self, File};
use std::io::{self, BufRead, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::str::FromStr;

pub struct FS {
    proc: Arc<Mutex<PathBuf>>,
    sys: Arc<Mutex<PathBuf>>,
}

impl FS {
    pub fn new_default_fs() -> io::Result<Self> {
        Self::new(fs::default_proc_mount_point(), fs::default_sys_mount_point())
    }

    pub fn new(proc_mount_point: PathBuf, sys_mount_point: PathBuf) -> io::Result<Self> {
        if proc_mount_point.exists() && sys_mount_point.exists() {
            Ok(FS {
                proc: Arc::new(Mutex::new(proc_mount_point)),
                sys: Arc::new(Mutex::new(sys_mount_point)),
            })
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Mount point can't be read",
            ))
        }
    }

    pub fn proc_diskstats(&self) -> io::Result<Vec<Diskstats>> {
        let proc_path = self.proc.lock().unwrap();
        let file = File::open(proc_path.join("diskstats"))?;
        parse_proc_diskstats(file)
    }

    pub fn sys_block_devices(&self) -> io::Result<Vec<String>> {
        let sys_path = self.sys.lock().unwrap();
        let device_dirs = fs::read_dir(sys_path.join("block"))?;
        let mut devices = Vec::new();
        for entry in device_dirs {
            let entry = entry?;
            devices.push(entry.file_name().into_string().unwrap());
        }
        Ok(devices)
    }

    pub fn sys_block_device_stat(&self, device: &str) -> io::Result<(IOStats, usize)> {
        let sys_path = self.sys.lock().unwrap();
        let data = fs::read_to_string(sys_path.join("block").join(device).join("stat"))?;
        parse_sys_block_device_stat(&data)
    }

    pub fn sys_block_device_queue_stats(&self, device: &str) -> io::Result<BlockQueueStats> {
        let sys_path = self.sys.lock().unwrap();
        let mut stat = BlockQueueStats::default();

        let files = vec![
            ("add_random", &mut stat.add_random),
            ("dax", &mut stat.dax),
            ("discard_granularity", &mut stat.discard_granularity),
            ("discard_max_hw_bytes", &mut stat.discard_max_hw_bytes),
            ("discard_max_bytes", &mut stat.discard_max_bytes),
            ("hw_sector_size", &mut stat.hw_sector_size),
            ("io_poll", &mut stat.io_poll),
            ("io_timeout", &mut stat.io_timeout),
            ("iostats", &mut stat.iostats),
            ("logical_block_size", &mut stat.logical_block_size),
            ("max_hw_sectors_kb", &mut stat.max_hw_sectors_kb),
            ("max_integrity_segments", &mut stat.max_integrity_segments),
            ("max_sectors_kb", &mut stat.max_sectors_kb),
            ("max_segments", &mut stat.max_segments),
            ("max_segment_size", &mut stat.max_segment_size),
            ("minimum_io_size", &mut stat.minimum_io_size),
            ("nomerges", &mut stat.nomerges),
            ("nr_requests", &mut stat.nr_requests),
            ("optimal_io_size", &mut stat.optimal_io_size),
            ("physical_block_size", &mut stat.physical_block_size),
            ("read_ahead_kb", &mut stat.read_ahead_kb),
            ("rotational", &mut stat.rotational),
            ("rq_affinity", &mut stat.rq_affinity),
            ("write_same_max_bytes", &mut stat.write_same_max_bytes),
            ("nr_zones", &mut stat.nr_zones),
            ("chunk_sectors", &mut stat.chunk_sectors),
            ("fua", &mut stat.fua),
            ("max_discard_segments", &mut stat.max_discard_segments),
            ("write_zeroes_max_bytes", &mut stat.write_zeroes_max_bytes),
        ];

        for (file, p) in files {
            let val = read_uint_from_file(&sys_path.join("block").join(device).join("queue").join(file))?;
            *p = val;
        }

        let int_files = vec![
            ("io_poll_delay", &mut stat.io_poll_delay),
            ("wbt_lat_usec", &mut stat.wbt_lat_usec),
        ];

        for (file, p) in int_files {
            let val = read_int_from_file(&sys_path.join("block").join(device).join("queue").join(file))?;
            *p = val;
        }

        let string_files = vec![
            ("write_cache", &mut stat.write_cache),
            ("zoned", &mut stat.zoned),
        ];

        for (file, p) in string_files {
            let val = read_string_from_file(&sys_path.join("block").join(device).join("queue").join(file))?;
            *p = val;
        }

        let scheduler = read_string_from_file(&sys_path.join("block").join(device).join("queue").join("scheduler"))?;
        let schedulers: Vec<String> = scheduler.split_whitespace().map(|s| s.to_string()).collect();
        for s in &schedulers {
            if s.starts_with('[') && s.ends_with(']') {
                stat.scheduler_current = s[1..s.len() - 1].to_string();
            }
        }
        stat.scheduler_list = schedulers;

        if let Ok(throttle_sample_time) = read_uint_from_file(&sys_path.join("block").join(device).join("queue").join("throttle_sample_time")) {
            stat.throttle_sample_time = Some(throttle_sample_time);
        }

        Ok(stat)
    }

    pub fn sys_block_device_mapper_info(&self, device: &str) -> io::Result<DeviceMapperInfo> {
        let sys_path = self.sys.lock().unwrap();
        let mut info = DeviceMapperInfo::default();

        let files = vec![
            ("rq_based_seq_io_merge_deadline", &mut info.rq_based_seq_io_merge_deadline),
            ("suspended", &mut info.suspended),
            ("use_blk_mq", &mut info.use_blk_mq),
        ];

        for (file, p) in files {
            let val = read_uint_from_file(&sys_path.join("block").join(device).join("dm").join(file))?;
            *p = val;
        }

        let string_files = vec![
            ("name", &mut info.name),
            ("uuid", &mut info.uuid),
        ];

        for (file, p) in string_files {
            let val = read_string_from_file(&sys_path.join("block").join(device).join("dm").join(file))?;
            *p = val;
        }

        Ok(info)
    }

    pub fn sys_block_device_underlying_devices(&self, device: &str) -> io::Result<UnderlyingDeviceInfo> {
        let sys_path = self.sys.lock().unwrap();
        let underlying_dir = fs::read_dir(sys_path.join("block").join(device).join("slaves"))?;
        let mut underlying = Vec::new();
        for entry in underlying_dir {
            let entry = entry?;
            underlying.push(entry.file_name().into_string().unwrap());
        }
        Ok(UnderlyingDeviceInfo { device_names: underlying })
    }

    pub fn sys_block_device_size(&self, device: &str) -> io::Result<u64> {
        let sys_path = self.sys.lock().unwrap();
        let size = read_uint_from_file(&sys_path.join("block").join(device).join("size"))?;
        Ok(procfs::SECTOR_SIZE * size)
    }

    pub fn sys_block_device_io_stat(&self, device: &str) -> io::Result<IODeviceStats> {
        let sys_path = self.sys.lock().unwrap();
        let mut io_device_stats = IODeviceStats::default();

        let files = vec![
            ("iodone_cnt", &mut io_device_stats.io_done_count),
            ("ioerr_cnt", &mut io_device_stats.io_err_count),
        ];

        for (file, p) in files {
            let val = read_hex_from_file(&sys_path.join("block").join(device).join("device").join(file))?;
            *p = val;
        }

        Ok(io_device_stats)
    }
}

#[derive(Default)]
pub struct IODeviceStats {
    pub io_done_count: u64,
    pub io_err_count: u64,
}

#[derive(Default)]
pub struct Diskstats {
    pub info: Info,
    pub io_stats: IOStats,
    pub io_stats_count: usize,
}

#[derive(Default)]
pub struct Info {
    pub major_number: u32,
    pub minor_number: u32,
    pub device_name: String,
}

#[derive(Default)]
pub struct IOStats {
    pub read_ios: u64,
    pub read_merges: u64,
    pub read_sectors: u64,
    pub read_ticks: u64,
    pub write_ios: u64,
    pub write_merges: u64,
    pub write_sectors: u64,
    pub write_ticks: u64,
    pub ios_in_progress: u64,
    pub ios_total_ticks: u64,
    pub weighted_io_ticks: u64,
    pub discard_ios: u64,
    pub discard_merges: u64,
    pub discard_sectors: u64,
    pub discard_ticks: u64,
    pub flush_requests_completed: u64,
    pub time_spent_flushing: u64,
}

#[derive(Default)]
pub struct BlockQueueStats {
    pub add_random: u64,
    pub dax: u64,
    pub discard_granularity: u64,
    pub discard_max_hw_bytes: u64,
    pub discard_max_bytes: u64,
    pub hw_sector_size: u64,
    pub io_poll: u64,
    pub io_poll_delay: i64,
    pub io_timeout: u64,
    pub iostats: u64,
    pub logical_block_size: u64,
    pub max_hw_sectors_kb: u64,
    pub max_integrity_segments: u64,
    pub max_sectors_kb: u64,
    pub max_segments: u64,
    pub max_segment_size: u64,
    pub minimum_io_size: u64,
    pub nomerges: u64,
    pub nr_requests: u64,
    pub optimal_io_size: u64,
    pub physical_block_size: u64,
    pub read_ahead_kb: u64,
    pub rotational: u64,
    pub rq_affinity: u64,
    pub write_cache: String,
    pub write_same_max_bytes: u64,
    pub wbt_lat_usec: i64,
    pub throttle_sample_time: Option<u64>,
    pub zoned: String,
    pub nr_zones: u64,
    pub chunk_sectors: u64,
    pub fua: u64,
    pub max_discard_segments: u64,
    pub write_zeroes_max_bytes: u64,
    pub scheduler_list: Vec<String>,
    pub scheduler_current: String,
}

#[derive(Default)]
pub struct DeviceMapperInfo {
    pub name: String,
    pub rq_based_seq_io_merge_deadline: u64,
    pub suspended: u64,
    pub use_blk_mq: u64,
    pub uuid: String,
}

#[derive(Default)]
pub struct UnderlyingDeviceInfo {
    pub device_names: Vec<String>,
}

mod fs {
    use std::path::PathBuf;

    pub fn default_proc_mount_point() -> PathBuf {
        PathBuf::from("/proc")
    }

    pub fn default_sys_mount_point() -> PathBuf {
        PathBuf::from("/sys")
    }
}

mod procfs {
    pub const SECTOR_SIZE: u64 = 512;
}

fn read_uint_from_file(path: &Path) -> io::Result<u64> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    contents.trim().parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn read_int_from_file(path: &Path) -> io::Result<i64> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    contents.trim().parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn read_string_from_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents.trim().to_string())
}

fn read_hex_from_file(path: &Path) -> io::Result<u64> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    u64::from_str_radix(contents.trim(), 16).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn parse_proc_diskstats<R: BufRead>(reader: R) -> io::Result<Vec<Diskstats>> {
    let mut diskstats = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let mut d = Diskstats::default();
        d.io_stats_count = sscanf::sscanf!(
            &line,
            "%d %d %s %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d",
            d.info.major_number,
            d.info.minor_number,
            d.info.device_name,
            d.io_stats.read_ios,
            d.io_stats.read_merges,
            d.io_stats.read_sectors,
            d.io_stats.read_ticks,
            d.io_stats.write_ios,
            d.io_stats.write_merges,
            d.io_stats.write_sectors,
            d.io_stats.write_ticks,
            d.io_stats.ios_in_progress,
            d.io_stats.ios_total_ticks,
            d.io_stats.weighted_io_ticks,
            d.io_stats.discard_ios,
            d.io_stats.discard_merges,
            d.io_stats.discard_sectors,
            d.io_stats.discard_ticks,
            d.io_stats.flush_requests_completed,
            d.io_stats.time_spent_flushing
        )
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if d.io_stats_count >= 14 {
            diskstats.push(d);
        }
    }
    Ok(diskstats)
}

fn parse_sys_block_device_stat(data: &str) -> io::Result<(IOStats, usize)> {
    let mut stat = IOStats::default();
    let count = sscanf::sscanf!(
        data.trim(),
        "%d %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d",
        stat.read_ios,
        stat.read_merges,
        stat.read_sectors,
        stat.read_ticks,
        stat.write_ios,
        stat.write_merges,
        stat.write_sectors,
        stat.write_ticks,
        stat.ios_in_progress,
        stat.ios_total_ticks,
        stat.weighted_io_ticks,
        stat.discard_ios,
        stat.discard_merges,
        stat.discard_sectors,
        stat.discard_ticks,
        stat.flush_requests_completed,
        stat.time_spent_flushing
    )
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok((stat, count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use super::*;
    use std::path::PathBuf;

    const FAIL_MSG_FORMAT: &str = "{}: expected {}, actual {}";
    const PROCFS_FIXTURES: &str = "testdata/fixtures/proc";
    const SYSFS_FIXTURES: &str = "testdata/fixtures/sys";

    #[test]
    fn test_diskstats() {
        let blockdevice = FS::new(PathBuf::from(PROCFS_FIXTURES), PathBuf::from(SYSFS_FIXTURES))
            .expect("failed to access blockdevice fs");
        let diskstats = blockdevice.proc_diskstats().expect("failed to parse diskstats");

        let expected_num_of_devices = 52;
        assert_eq!(diskstats.len(), expected_num_of_devices, FAIL_MSG_FORMAT, "Incorrect number of devices", expected_num_of_devices, diskstats.len());
        assert_eq!(diskstats[0].info.device_name, "ram0", FAIL_MSG_FORMAT, "Incorrect device name", "ram0", diskstats[0].info.device_name);
        assert_eq!(diskstats[1].io_stats_count, 14, FAIL_MSG_FORMAT, "Incorrect number of stats read", 14, diskstats[1].io_stats_count);
        assert_eq!(diskstats[24].io_stats.write_ios, 28444756, FAIL_MSG_FORMAT, "Incorrect writes completed", 28444756, diskstats[24].io_stats.write_ios);
        assert_eq!(diskstats[48].io_stats.discard_ticks, 11130, FAIL_MSG_FORMAT, "Incorrect discard time", 11130, diskstats[48].io_stats.discard_ticks);
        assert_eq!(diskstats[48].io_stats_count, 18, FAIL_MSG_FORMAT, "Incorrect number of stats read", 18, diskstats[48].io_stats_count);
        assert_eq!(diskstats[49].io_stats_count, 20, FAIL_MSG_FORMAT, "Incorrect number of stats read", 20, diskstats[49].io_stats_count);
        assert_eq!(diskstats[49].io_stats.flush_requests_completed, 127, FAIL_MSG_FORMAT, "Incorrect number of flush requests completed", 127, diskstats[49].io_stats.flush_requests_completed);
        assert_eq!(diskstats[49].io_stats.time_spent_flushing, 182, FAIL_MSG_FORMAT, "Incorrect time spent flushing", 182, diskstats[49].io_stats.time_spent_flushing);
    }

    #[test]
    fn test_block_device() {
        let blockdevice = FS::new(PathBuf::from(PROCFS_FIXTURES), PathBuf::from(SYSFS_FIXTURES))
            .expect("failed to access blockdevice fs");
        let devices = blockdevice.sys_block_devices().expect("failed to get block devices");

        let expected_num_of_devices = 8;
        assert_eq!(devices.len(), expected_num_of_devices, FAIL_MSG_FORMAT, "Incorrect number of devices", expected_num_of_devices, devices.len());
        assert_eq!(devices[0], "dm-0", FAIL_MSG_FORMAT, "Incorrect device name", "dm-0", devices[0]);

        let (device0stats, count) = blockdevice.sys_block_device_stat(&devices[0]).expect("failed to get device stats");
        assert_eq!(count, 11, FAIL_MSG_FORMAT, "Incorrect number of stats read", 11, count);
        assert_eq!(device0stats.read_ios, 6447303, FAIL_MSG_FORMAT, "Incorrect read I/Os", 6447303, device0stats.read_ios);
        assert_eq!(device0stats.weighted_io_ticks, 6088971, FAIL_MSG_FORMAT, "Incorrect time in queue", 6088971, device0stats.weighted_io_ticks);

        let (device7stats, count) = blockdevice.sys_block_device_stat(&devices[7]).expect("failed to get device stats");
        assert_eq!(count, 15, FAIL_MSG_FORMAT, "Incorrect number of stats read", 15, count);
        assert_eq!(device7stats.write_sectors, 286915323, FAIL_MSG_FORMAT, "Incorrect write merges", 286915323, device7stats.write_sectors);
        assert_eq!(device7stats.discard_ticks, 12, FAIL_MSG_FORMAT, "Incorrect discard ticks", 12, device7stats.discard_ticks);

        let block_queue_stat_expected = BlockQueueStats {
            add_random: 1,
            dax: 0,
            discard_granularity: 0,
            discard_max_hw_bytes: 0,
            discard_max_bytes: 0,
            hw_sector_size: 512,
            io_poll: 0,
            io_poll_delay: -1,
            io_timeout: 30000,
            iostats: 1,
            logical_block_size: 512,
            max_hw_sectors_kb: 32767,
            max_integrity_segments: 0,
            max_sectors_kb: 1280,
            max_segments: 168,
            max_segment_size: 65536,
            minimum_io_size: 512,
            nomerges: 0,
            nr_requests: 64,
            optimal_io_size: 0,
            physical_block_size: 512,
            read_ahead_kb: 128,
            rotational: 1,
            rq_affinity: 1,
            write_cache: "write back".to_string(),
            write_same_max_bytes: 0,
            wbt_lat_usec: 75000,
            throttle_sample_time: None,
            zoned: "none".to_string(),
            nr_zones: 0,
            chunk_sectors: 0,
            fua: 0,
            max_discard_segments: 1,
            write_zeroes_max_bytes: 0,
            scheduler_list: vec!["mq-deadline".to_string(), "kyber".to_string(), "bfq".to_string(), "none".to_string()],
            scheduler_current: "bfq".to_string(),
        };

        let block_queue_stat = blockdevice.sys_block_device_queue_stats(&devices[7]).expect("failed to get block queue stats");
        assert_eq!(block_queue_stat, block_queue_stat_expected, "Incorrect BlockQueueStat, expected: \n{:?}, got: \n{:?}", block_queue_stat_expected, block_queue_stat);
    }

    #[test]
    fn test_block_dm_info() {
        let blockdevice = FS::new(PathBuf::from(PROCFS_FIXTURES), PathBuf::from(SYSFS_FIXTURES))
            .expect("failed to access blockdevice fs");
        let devices = blockdevice.sys_block_devices().expect("failed to get block devices");

        let dm0_info = blockdevice.sys_block_device_mapper_info(&devices[0]).expect("failed to get device mapper info");

        let dm0_info_expected = DeviceMapperInfo {
            name: "vg0--lv_root".to_string(),
            rq_based_seq_io_merge_deadline: 0,
            suspended: 0,
            use_blk_mq: 0,
            uuid: "LVM-3zSHSR5Nbf4j7g6auAAefWY2CMaX01theZYEvQyecVsm2WtX3iY5q51qq5dWWOq7".to_string(),
        };
        assert_eq!(dm0_info, dm0_info_expected, "Incorrect BlockQueueStat, expected: \n{:?}, got: \n{:?}", dm0_info_expected, dm0_info);

        let dm1_info = blockdevice.sys_block_device_mapper_info(&devices[1]);
        match dm1_info {
            Err(e) => {
                if e.kind() != io::ErrorKind::NotFound {
                    panic!("Unexpected error: {:?}", e);
                }
            }
            Ok(_) => panic!("SysBlockDeviceMapperInfo on sda was supposed to fail."),
        }
    }

    #[test]
    fn test_sys_block_device_underlying_devices() {
        let blockdevice = FS::new(PathBuf::from(PROCFS_FIXTURES), PathBuf::from(SYSFS_FIXTURES))
            .expect("failed to access blockdevice fs");
        let devices = blockdevice.sys_block_devices().expect("failed to get block devices");

        let underlying0 = blockdevice.sys_block_device_underlying_devices(&devices[0]).expect("failed to get underlying devices");
        let underlying0_expected = UnderlyingDeviceInfo {
            device_names: vec!["sda".to_string()],
        };
        assert_eq!(underlying0, underlying0_expected, "Incorrect BlockQueueStat, expected: \n{:?}, got: \n{:?}", underlying0_expected, underlying0);
    }

    #[test]
    fn test_sys_block_device_size() {
        let blockdevice = FS::new(PathBuf::from(PROCFS_FIXTURES), PathBuf::from(SYSFS_FIXTURES))
            .expect("failed to access blockdevice fs");
        let devices = blockdevice.sys_block_devices().expect("failed to get block devices");

        let size7 = blockdevice.sys_block_device_size(&devices[7]).expect("failed to get block device size");
        let size7_expected = 1920383410176;
        assert_eq!(size7, size7_expected, "Incorrect BlockDeviceSize, expected: \n{:?}, got: \n{:?}", size7_expected, size7);
    }

    #[test]
    fn test_sys_block_device_io_stat() {
        let fs = FS::new(PathBuf::from("testdata/fixtures/proc"), PathBuf::from("testdata/fixtures/sys")).expect("failed to access fs");
        let stats = fs.sys_block_device_io_stat("sda").expect("failed to get io stats");

        assert_eq!(stats.io_done_count, 12345); // Replace with expected value
        assert_eq!(stats.io_err_count, 678); // Replace with expected value
    }
}