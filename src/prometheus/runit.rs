use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

const DEFAULT_SERVICE_DIR: &str = "/etc/service";
const TAI_OFFSET: i64 = 4611686018427387914;
const STATUS_LEN: usize = 20;

const POS_TIME_START: usize = 0;
const POS_TIME_END: usize = 7;
const POS_PID_START: usize = 12;
const POS_PID_END: usize = 15;

const POS_WANT: usize = 17;
const POS_STATE: usize = 19;

const STATE_DOWN: u8 = 0;
const STATE_UP: u8 = 1;
const STATE_FINISH: u8 = 2;

lazy_static::lazy_static! {
    static ref STATE_TO_STRING: HashMap<u8, &'static str> = {
        let mut m = HashMap::new();
        m.insert(STATE_DOWN, "down");
        m.insert(STATE_UP, "up");
        m.insert(STATE_FINISH, "finish");
        m
    };
}

#[derive(Debug)]
pub struct SvStatus {
    pid: i32,
    duration: i32,
    timestamp: SystemTime,
    state: u8,
    normally_up: bool,
    want: u8,
}

#[derive(Debug)]
pub struct Service {
    name: String,
    service_dir: String,
}

impl Service {
    pub fn new(name: &str, dir: &str) -> Self {
        Self {
            name: name.to_string(),
            service_dir: if dir.is_empty() { DEFAULT_SERVICE_DIR.to_string() } else { dir.to_string() },
        }
    }

    fn file(&self, file: &str) -> String {
        format!("{}/{}/supervise/{}", self.service_dir, self.name, file)
    }

    fn status(&self) -> io::Result<Vec<u8>> {
        let mut file = File::open(self.file("status"))?;
        let mut status = vec![0; STATUS_LEN];
        file.read_exact(&mut status)?;
        Ok(status)
    }

    pub fn normally_up(&self) -> bool {
        fs::metadata(self.file("down")).is_err()
    }

    pub fn get_status(&self) -> io::Result<SvStatus> {
        let status = self.status()?;

        let pid = status[POS_PID_START..=POS_PID_END]
            .iter()
            .fold(0, |acc, &b| (acc << 8) + b as i32);

        let tai = status[POS_TIME_START..=POS_TIME_END]
            .iter()
            .fold(0, |acc, &b| (acc << 8) + b as i64);

        let state = status[POS_STATE];

        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH + Duration::from_secs((tai - TAI_OFFSET) as u64))
            .unwrap_or_default()
            .as_secs() as i32;

        let want = match status[POS_WANT] {
            b'u' => STATE_UP,
            b'd' => STATE_DOWN,
            _ => STATE_DOWN,
        };

        Ok(SvStatus {
            pid,
            duration,
            timestamp: UNIX_EPOCH + Duration::from_secs((tai - TAI_OFFSET) as u64),
            state,
            normally_up: self.normally_up(),
            want,
        })
    }
}

pub fn get_services(dir: &str) -> io::Result<Vec<Service>> {
    let dir = if dir.is_empty() { DEFAULT_SERVICE_DIR } else { dir };
    let mut services = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_symlink() || entry.file_type()?.is_dir() {
            services.push(Service::new(entry.file_name().to_str().unwrap(), dir));
        }
    }
    Ok(services)
}