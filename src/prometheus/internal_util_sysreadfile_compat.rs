// Copyright 2019 The Prometheus Authors
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

#[cfg(any(target_os = "windows", all(target_os = "linux", target_env = "sgx")))]
pub fn sys_read_file(file: &str) -> Result<String, std::io::Error> {
    Err(std::io::Error::new(std::io::ErrorKind::Other, "not supported on this platform"))
}

// fn main() {
//     match sys_read_file("somefile.txt") {
//         Ok(contents) => println!("File contents: {}", contents),
//         Err(e) => println!("Error: {}", e),
//     }
// }