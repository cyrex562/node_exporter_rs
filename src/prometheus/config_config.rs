use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json;
use serde_yaml;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const SECRET_TOKEN: &str = "<secret>";

static MARSHAL_SECRET_VALUE: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug, Default)]
pub struct Secret(pub String);

impl Serialize for Secret {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if MARSHAL_SECRET_VALUE.load(Ordering::Relaxed) {
            serializer.serialize_str(&self.0)
        } else if !self.0.is_empty() {
            serializer.serialize_str(SECRET_TOKEN)
        } else {
            serializer.serialize_none()
        }
    }
}

impl<'de> Deserialize<'de> for Secret {
    fn deserialize<D>(deserializer: D) -> Result<Secret, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Secret(s))
    }
}

pub fn set_marshal_secret_value(value: bool) {
    MARSHAL_SECRET_VALUE.store(value, Ordering::Relaxed);
}

pub type ProxyHeader = HashMap<String, Vec<Secret>>;

impl ProxyHeader {
    pub fn http_header(&self) -> Option<http::HeaderMap> {
        if self.is_empty() {
            return None;
        }

        let mut header = http::HeaderMap::new();

        for (name, values) in self {
            if !values.is_empty() {
                for value in values {
                    if let Ok(header_name) = http::header::HeaderName::from_bytes(name.as_bytes()) {
                        let header_value = http::HeaderValue::from_str(&value.0).unwrap_or_default();
                        header.append(header_name.clone(), header_value);
                    }
                }
            }
        }

        Some(header)
    }
}

pub trait DirectorySetter {
    fn set_directory(&mut self, dir: &Path);
}

pub fn join_dir(dir: &Path, path: &Path) -> PathBuf {
    if path.as_os_str().is_empty() || path.is_absolute() {
        path.to_path_buf()
    } else {
        dir.join(path)
    }
}