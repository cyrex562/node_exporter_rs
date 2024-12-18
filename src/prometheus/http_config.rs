use std::time::Duration;
use std::collections::HashMap;
use tokio_rustls::rustls::version::{TLS13, TLS12, TLS11, TLS10};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json;
use serde_yaml;
use std::fmt;
use url::Url;

#[derive(Debug, Clone)]
pub struct HTTPClientConfig {
    pub follow_redirects: bool,
    pub enable_http2: bool,
}

impl Default for HTTPClientConfig {
    fn default() -> Self {
        HTTPClientConfig {
            follow_redirects: true,
            enable_http2: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpClientOptions {
    pub keep_alives_enabled: bool,
    pub http2_enabled: bool,
    pub idle_conn_timeout: Duration,
}

impl Default for HttpClientOptions {
    fn default() -> Self {
        HttpClientOptions {
            keep_alives_enabled: true,
            http2_enabled: true,
            idle_conn_timeout: Duration::from_secs(5 * 60), // 5 minutes
        }
    }
}

lazy_static::lazy_static! {
    // Default HTTP client configuration.
    pub static ref DEFAULT_HTTP_CLIENT_CONFIG: HTTPClientConfig = HTTPClientConfig::default();

    // Default HTTP client options.
    pub static ref DEFAULT_HTTP_CLIENT_OPTIONS: HttpClientOptions = HttpClientOptions::default();
}

pub trait CloseIdler {
    fn close_idle_connections(&self);
}

type TLSVersion = u16;

lazy_static::lazy_static! {
    pub static ref TLS_VERSIONS: HashMap<&'static str, u16> = {
        let mut m = HashMap::new();
        m.insert("TLS13", TLS13);
        m.insert("TLS12", TLS12);
        m.insert("TLS11", TLS11);
        m.insert("TLS10", TLS10);
        m
    };
}

impl<'de> Deserialize<'de> for TLSVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        TLS_VERSIONS
            .get(s.as_str())
            .cloned()
            .ok_or_else(|| serde::de::Error::custom(format!("unknown TLS version: {}", s)))
    }
}

impl Serialize for TLSVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        for (s, &v) in TLS_VERSIONS.iter() {
            if *self == v {
                return serializer.serialize_str(s);
            }
        }
        Err(serde::ser::Error::custom(format!("unknown TLS version: {}", self)))
    }
}

impl<'de> Deserialize<'de> for TLSVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        TLS_VERSIONS
            .get(s.as_str())
            .cloned()
            .ok_or_else(|| serde::de::Error::custom(format!("unknown TLS version: {}", s)))
    }
}

impl Serialize for TLSVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        for (s, &v) in TLS_VERSIONS.iter() {
            if *self == v {
                return serializer.serialize_str(s);
            }
        }
        Err(serde::ser::Error::custom(format!("unknown TLS version: {}", self)))
    }
}

impl fmt::Display for TLSVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (s, &v) in TLS_VERSIONS.iter() {
            if *self == v {
                return write!(f, "{}", s);
            }
        }
        write!(f, "{}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuth {
    pub username: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<Secret>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password_ref: Option<String>,
}

impl BasicAuth {
    // Joins any relative file paths with dir.
    pub fn set_directory(&mut self, dir: &Path) {
        if let Some(ref mut password_file) = self.password_file {
            *password_file = join_dir(dir, Path::new(password_file)).to_string_lossy().into_owned();
        }
        if let Some(ref mut username_file) = self.username_file {
            *username_file = join_dir(dir, Path::new(username_file)).to_string_lossy().into_owned();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret(String);

impl From<String> for Secret {
    fn from(s: String) -> Self {
        Secret(s)
    }
}

fn join_dir(dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        dir.join(path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authorization {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credentials: Option<Secret>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credentials_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credentials_ref: Option<String>,
}

impl Authorization {
    // Joins any relative file paths with dir.
    pub fn set_directory(&mut self, dir: &Path) {
        if let Some(ref mut credentials_file) = self.credentials_file {
            *credentials_file = join_dir(dir, Path::new(credentials_file)).to_string_lossy().into_owned();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret(String);

impl From<String> for Secret {
    fn from(s: String) -> Self {
        Secret(s)
    }
}

fn join_dir(dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        dir.join(path)
    }
}

#[derive(Debug, Clone)]
pub struct URL {
    pub url: Option<Url>,
}

impl URL {
    // Redacted returns the URL but replaces any password with "xxxxx".
    pub fn redacted(&self) -> String {
        if let Some(ref url) = self.url {
            let mut redacted_url = url.clone();
            if redacted_url.password().is_some() {
                redacted_url.set_password(Some("xxxxx")).unwrap();
            }
            redacted_url.to_string()
        } else {
            String::new()
        }
    }
}

impl<'de> Deserialize<'de> for URL {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let url = Url::parse(&s).map_err(serde::de::Error::custom)?;
        Ok(URL { url: Some(url) })
    }
}

impl Serialize for URL {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(ref url) = self.url {
            serializer.serialize_str(&self.redacted())
        } else {
            serializer.serialize_none()
        }
    }
}

impl<'de> Deserialize<'de> for URL {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let url = Url::parse(&s).map_err(serde::de::Error::custom)?;
        Ok(URL { url: Some(url) })
    }
}

impl Serialize for URL {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(ref url) = self.url {
            serializer.serialize_str(&self.redacted())
        } else {
            serializer.serialize_none()
        }
    }
}

impl fmt::Display for URL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref url) = self.url {
            write!(f, "{}", url)
        } else {
            write!(f, "")
        }
    }
}