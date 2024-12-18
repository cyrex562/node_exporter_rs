// Copyright 2018 The Prometheus Authors
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

#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::fs::File;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::io::Error;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::unix::io::AsRawFd;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::ptr;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::slice;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn sys_read_file(file: &str) -> Result<String, Error> {
    let f = File::open(file)?;
    let fd = f.as_raw_fd();

    const SYS_FILE_BUFFER_SIZE: usize = 128;
    let mut buffer = [0u8; SYS_FILE_BUFFER_SIZE];

    let n = unsafe {
        let ret = libc::read(fd, buffer.as_mut_ptr() as *mut libc::c_void, SYS_FILE_BUFFER_SIZE);
        if ret < 0 {
            return Err(Error::last_os_error());
        }
        ret as usize
    };

    let content = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
    Ok(content)
}

// fn main() {
//     match sys_read_file("somefile.txt") {
//         Ok(contents) => println!("File contents: {}", contents),
//         Err(e) => println!("Error: {}", e),
//     }
// }