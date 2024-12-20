use std::fs;
use std::path::Path;
use std::io::ErrorKind;

pub struct KernelRandom {
    entropy_available: Option<u64>,
    pool_size: Option<u64>,
    urandom_min_reseed_seconds: Option<u64>,
    write_wakeup_threshold: Option<u64>,
    read_wakeup_threshold: Option<u64>,
}

pub struct ProcFs {
    proc: String,
}

impl ProcFs {
    pub fn new(proc: &str) -> Self {
        ProcFs { proc: proc.to_string() }
    }

    pub fn kernel_random(&self) -> Result<KernelRandom, std::io::Error> {
        let mut random = KernelRandom {
            entropy_available: None,
            pool_size: None,
            urandom_min_reseed_seconds: None,
            write_wakeup_threshold: None,
            read_wakeup_threshold: None,
        };

        let files = vec![
            ("entropy_avail", &mut random.entropy_available),
            ("poolsize", &mut random.pool_size),
            ("urandom_min_reseed_secs", &mut random.urandom_min_reseed_seconds),
            ("write_wakeup_threshold", &mut random.write_wakeup_threshold),
            ("read_wakeup_threshold", &mut random.read_wakeup_threshold),
        ];

        for (file, field) in files {
            match read_uint_from_file(&self.proc, file) {
                Ok(val) => *field = Some(val),
                Err(e) if e.kind() == ErrorKind::NotFound => continue,
                Err(e) => return Err(e),
            }
        }

        Ok(random)
    }
}

fn read_uint_from_file(proc: &str, file: &str) -> Result<u64, std::io::Error> {
    let path = Path::new(proc).join("sys/kernel/random").join(file);
    let content = fs::read_to_string(path)?;
    content.trim().parse().map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))
}