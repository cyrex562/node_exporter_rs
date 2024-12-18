use std::fs::File as StdFile;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::Arc;
use std::sync::Mutex;
use std::fs::Metadata;
use std::path::Path;
use std::fs;

pub struct File {
    file: Arc<Mutex<StdFile>>,
    content: Vec<u8>,
    offset: usize,
}

impl File {
    pub fn new(file: StdFile, content: Vec<u8>) -> Self {
        File {
            file: Arc::new(Mutex::new(file)),
            content,
            offset: 0,
        }
    }

    pub fn stat(&self) -> io::Result<FileInfo> {
        let file = self.file.lock().unwrap();
        let metadata = file.metadata()?;
        Ok(FileInfo {
            metadata,
            actual_size: self.content.len() as u64,
        })
    }

    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let remaining = self.content.len() - self.offset;
        let to_read = buf.len().min(remaining);
        buf[..to_read].copy_from_slice(&self.content[self.offset..self.offset + to_read]);
        self.offset += to_read;
        if to_read == remaining {
            Ok(to_read)
        } else {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"))
        }
    }

    pub fn close(&self) -> io::Result<()> {
        let file = self.file.lock().unwrap();
        file.sync_all()
    }
}

pub struct FileInfo {
    metadata: Metadata,
    actual_size: u64,
}

impl FileInfo {
    pub fn name(&self) -> String {
        let name = self.metadata.file_name().into_string().unwrap();
        let gzip_suffix = ".gz";
        name[..name.len() - gzip_suffix.len()].to_string()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File as StdFile;
    use std::io::Read;
    use std::path::Path;

    fn setup_test_file(path: &str, content: &[u8]) -> File {
        let file = StdFile::open(path).unwrap();
        File::new(file, content.to_vec())
    }

    #[test]
    fn test_fs() {
        let cases = vec![
            ("uncompressed file", "testdata/uncompressed", 4, "test"),
            ("compressed file", "testdata/compressed.gz", 4, "test"),
        ];

        for (name, path, expected_size, expected_content) in cases {
            let mut file = setup_test_file(path, expected_content.as_bytes());
            let mut buf = vec![0; expected_size as usize];
            let size = file.read(&mut buf).unwrap();
            assert_eq!(size, expected_size as usize);
            assert_eq!(String::from_utf8(buf).unwrap(), expected_content);
        }
    }

    #[test]
    fn test_stat() {
        let path = "testdata/uncompressed";
        let content = b"test";
        let file = setup_test_file(path, content);
        let file_info = file.stat().unwrap();
        assert_eq!(file_info.actual_size, content.len() as u64);
    }

    #[test]
    fn test_close() {
        let path = "testdata/uncompressed";
        let content = b"test";
        let file = setup_test_file(path, content);
        assert!(file.close().is_ok());
    }
}