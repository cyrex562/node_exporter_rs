use std::fs;
use std::io;
use std::path::Path;

struct FS {
    proc: PathBuf,
}

impl FS {
    fn cmd_line(&self) -> Result<Vec<String>, io::Error> {
        let data = fs::read_to_string(self.proc.join("cmdline"))?;
        Ok(data.split_whitespace().map(String::from).collect())
    }
}