// Copyright 2016 The Prometheus Authors
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

use std::fmt;
use std::sync::Once;
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use once_cell::sync::Lazy;

static INIT: Once = Once::new();
static VERSION_INFO: Lazy<Mutex<VersionInfo>> = Lazy::new(|| Mutex::new(VersionInfo::default()));

#[derive(Default)]
struct VersionInfo {
    version: String,
    revision: String,
    branch: String,
    build_user: String,
    build_date: String,
    go_version: String,
    go_os: String,
    go_arch: String,
    computed_revision: String,
    computed_tags: String,
}

impl VersionInfo {
    fn new() -> Self {
        let mut vi = VersionInfo::default();
        vi.go_version = env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string());
        vi.go_os = env::consts::OS.to_string();
        vi.go_arch = env::consts::ARCH.to_string();
        vi
    }

    fn compute_revision(&mut self) {
        let build_info = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string());
        let mut modified = false;
        let mut rev = "unknown".to_string();
        let mut tags = "unknown".to_string();

        if let Ok(build_info) = env::var("CARGO_PKG_VERSION") {
            for (key, value) in build_info.split(',').map(|s| {
                let mut parts = s.split('=');
                (parts.next().unwrap_or(""), parts.next().unwrap_or(""))
            }) {
                match key {
                    "vcs.revision" => rev = value.to_string(),
                    "vcs.modified" => modified = value == "true",
                    "-tags" => tags = value.to_string(),
                    _ => {}
                }
            }
        }

        if modified {
            rev.push_str("-modified");
        }

        self.computed_revision = rev;
        self.computed_tags = tags;
    }
}

pub fn print(program: &str) -> String {
    INIT.call_once(|| {
        let mut vi = VERSION_INFO.lock().unwrap();
        *vi = VersionInfo::new();
        vi.compute_revision();
    });

    let vi = VERSION_INFO.lock().unwrap();
    let mut m = HashMap::new();
    m.insert("program", program);
    m.insert("version", &vi.version);
    m.insert("revision", &vi.revision);
    m.insert("branch", &vi.branch);
    m.insert("build_user", &vi.build_user);
    m.insert("build_date", &vi.build_date);
    m.insert("go_version", &vi.go_version);
    m.insert("platform", &format!("{}/{}", vi.go_os, vi.go_arch));
    m.insert("tags", &vi.computed_tags);

    let tmpl = r#"
{{program}}, version {{version}} (branch: {{branch}}, revision: {{revision}})
  build user:       {{build_user}}
  build date:       {{build_date}}
  go version:       {{go_version}}
  platform:         {{platform}}
  tags:             {{tags}}
"#;

    let mut output = String::new();
    for line in tmpl.lines() {
        let mut line = line.to_string();
        for (key, value) in &m {
            line = line.replace(&format!("{{{{{}}}}}", key), value);
        }
        output.push_str(&line);
        output.push('\n');
    }

    output.trim().to_string()
}

pub fn info() -> String {
    INIT.call_once(|| {
        let mut vi = VERSION_INFO.lock().unwrap();
        *vi = VersionInfo::new();
        vi.compute_revision();
    });

    let vi = VERSION_INFO.lock().unwrap();
    format!(
        "(version={}, branch={}, revision={})",
        vi.version, vi.branch, vi.revision
    )
}

pub fn build_context() -> String {
    INIT.call_once(|| {
        let mut vi = VERSION_INFO.lock().unwrap();
        *vi = VersionInfo::new();
        vi.compute_revision();
    });

    let vi = VERSION_INFO.lock().unwrap();
    format!(
        "(go={}, platform={}/{}, user={}, date={}, tags={})",
        vi.go_version, vi.go_os, vi.go_arch, vi.build_user, vi.build_date, vi.computed_tags
    )
}

pub fn prometheus_user_agent() -> String {
    component_user_agent("Prometheus")
}

pub fn component_user_agent(component: &str) -> String {
    INIT.call_once(|| {
        let mut vi = VERSION_INFO.lock().unwrap();
        *vi = VersionInfo::new();
        vi.compute_revision();
    });

    let vi = VERSION_INFO.lock().unwrap();
    format!("{}/{}", component, vi.version)
}