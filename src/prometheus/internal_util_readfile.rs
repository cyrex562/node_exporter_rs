use std::fs::File;
use std::io::{self, Read, Result};

const MAX_BUFFER_SIZE: usize = 1024 * 1024; // 1024kB

/// Reads the contents of an entire file without calling `metadata` to avoid issues with
/// incorrect file sizes in `/proc` and `/sys`. Reads a maximum file size of 1024kB.
/// For files larger than this, a different method should be used.
fn read_file_no_stat(filename: &str) -> Result<Vec<u8>> {
    let mut file = File::open(filename)?;
    let mut buffer = Vec::with_capacity(MAX_BUFFER_SIZE);
    let mut reader = file.take(MAX_BUFFER_SIZE as u64);
    reader.read_to_end(&mut buffer)?;
    Ok(buffer)
}
