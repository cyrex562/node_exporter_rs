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

use std::collections::HashMap;
use std::fs;
use std::io;
use std::net::{TcpListener, ToSocketAddrs};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::net::TcpListener as TokioTcpListener;
use tokio::sync::Mutex as AsyncMutex;
use tokio_rustls::rustls::{self, Certificate, PrivateKey, ServerConfig};
use tokio_rustls::TlsAcceptor;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct Config {
    pub tls_config: TLSConfig,
    pub http_config: HTTPConfig,
    pub users: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TLSConfig {
    pub tls_cert: String,
    pub tls_key: String,
    pub client_cas_text: String,
    pub tls_cert_path: String,
    pub tls_key_path: String,
    pub client_auth: String,
    pub client_cas: String,
    pub cipher_suites: Vec<Cipher>,
    pub curve_preferences: Vec<Curve>,
    pub min_version: TLSVersion,
    pub max_version: TLSVersion,
    pub prefer_server_cipher_suites: bool,
    pub client_allowed_sans: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HTTPConfig {
    pub http2: bool,
    pub header: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct FlagConfig {
    pub web_listen_addresses: Vec<String>,
    pub web_systemd_socket: bool,
    pub web_config_file: String,
}

impl TLSConfig {
    pub fn set_directory(&mut self, dir: &str) {
        self.tls_cert_path = join_dir(dir, &self.tls_cert_path);
        self.tls_key_path = join_dir(dir, &self.tls_key_path);
        self.client_cas = join_dir(dir, &self.client_cas);
    }

    pub fn verify_peer_certificate(&self, raw_certs: &[Vec<u8>]) -> Result<(), Box<dyn std::error::Error>> {
        let cert = rustls::Certificate(raw_certs[0].clone());
        let parsed_cert = rustls::internal::pemfile::certs(&mut &cert.0[..])?.remove(0);

        let san_values = parsed_cert
            .subject_alt_names()
            .iter()
            .flat_map(|san| san.general_names.iter())
            .map(|gn| gn.to_string())
            .collect::<Vec<_>>();

        for san_value in san_values {
            if self.client_allowed_sans.contains(&san_value) {
                return Ok(());
            }
        }

        Err(format!("could not find allowed SANs in client cert, found: {:?}", self.client_allowed_sans).into())
    }
}

fn join_dir(dir: &str, path: &str) -> String {
    if Path::new(path).is_relative() {
        Path::new(dir).join(path).to_string_lossy().to_string()
    } else {
        path.to_string()
    }
}

pub async fn get_config(config_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(config_path)?;
    let mut config: Config = serde_yaml::from_str(&content)?;
    config.tls_config.set_directory(Path::new(config_path).parent().unwrap().to_str().unwrap());
    validate_header_config(&config.http_config.header)?;
    Ok(config)
}

pub async fn get_tls_config(config_path: &str) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let config = get_config(config_path).await?;
    config_to_tls_config(&config.tls_config).await
}

pub async fn config_to_tls_config(config: &TLSConfig) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    validate_tls_paths(config)?;

    let certs = load_certs(&config.tls_cert_path, &config.tls_cert)?;
    let key = load_private_key(&config.tls_key_path, &config.tls_key)?;

    let mut tls_config = ServerConfig::new(rustls::NoClientAuth::new());
    tls_config.set_single_cert(certs, key)?;

    if !config.cipher_suites.is_empty() {
        tls_config.ciphersuites = config.cipher_suites.iter().map(|&c| c.into()).collect();
    }

    if !config.curve_preferences.is_empty() {
        tls_config.kx_groups = config.curve_preferences.iter().map(|&c| c.into()).collect();
    }

    if !config.client_cas.is_empty() {
        let client_ca_cert = load_certs(&config.client_cas, &config.client_cas_text)?;
        let mut client_ca_root = rustls::RootCertStore::empty();
        client_ca_root.add(&client_ca_cert[0])?;
        tls_config.client_auth_roots = client_ca_root;
    }

    if !config.client_allowed_sans.is_empty() {
        tls_config.client_cert_verifier = Arc::new(ClientCertVerifier {
            allowed_sans: config.client_allowed_sans.clone(),
        });
    }

    Ok(tls_config)
}

fn validate_tls_paths(config: &TLSConfig) -> Result<(), Box<dyn std::error::Error>> {
    if config.tls_cert_path.is_empty() && config.tls_cert.is_empty()
        && config.tls_key_path.is_empty() && config.tls_key.is_empty()
        && config.client_cas.is_empty() && config.client_cas_text.is_empty()
        && config.client_auth.is_empty()
    {
        return Err("TLS config is not present".into());
    }

    if config.tls_cert_path.is_empty() && config.tls_cert.is_empty() {
        return Err("missing one of cert or cert_file".into());
    }

    if config.tls_key_path.is_empty() && config.tls_key.is_empty() {
        return Err("missing one of key or key_file".into());
    }

    Ok(())
}

fn load_certs(path: &str, content: &str) -> Result<Vec<Certificate>, Box<dyn std::error::Error>> {
    if !path.is_empty() {
        let cert_data = fs::read(path)?;
        Ok(rustls::internal::pemfile::certs(&mut &cert_data[..])?)
    } else {
        Ok(rustls::internal::pemfile::certs(&mut content.as_bytes())?)
    }
}

fn load_private_key(path: &str, content: &str) -> Result<PrivateKey, Box<dyn std::error::Error>> {
    if !path.is_empty() {
        let key_data = fs::read(path)?;
        Ok(rustls::internal::pemfile::pkcs8_private_keys(&mut &key_data[..])?.remove(0))
    } else {
        Ok(rustls::internal::pemfile::pkcs8_private_keys(&mut content.as_bytes())?.remove(0))
    }
}

pub async fn serve_multiple(listeners: Vec<TokioTcpListener>, server: Arc<hyper::Server<impl hyper::service::Service<hyper::Request<hyper::Body>, Response = hyper::Response<hyper::Body>, Error = hyper::Error> + Clone + Send + 'static>>, flags: FlagConfig, logger: slog::Logger) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = Vec::new();
    let cancel_token = CancellationToken::new();

    for listener in listeners {
        let server = server.clone();
        let cancel_token = cancel_token.clone();
        let logger = logger.clone();

        tasks.push(tokio::spawn(async move {
            let mut incoming = TcpListenerStream::new(listener);
            while let Some(stream) = incoming.next().await {
                match stream {
                    Ok(stream) => {
                        let server = server.clone();
                        let logger = logger.clone();
                        tokio::spawn(async move {
                            if let Err(err) = server.serve_connection(stream, hyper::service::service_fn(|req| async move {
                                Ok::<_, hyper::Error>(hyper::Response::new(hyper::Body::from("Hello World")))
                            })).await {
                                logger.error("Error serving connection", slog::o!("error" => err.to_string()));
                            }
                        });
                    }
                    Err(err) => {
                        logger.error("Error accepting connection", slog::o!("error" => err.to_string()));
                    }
                }
            }
        }));
    }

    cancel_token.cancelled().await;
    Ok(())
}

pub async fn listen_and_serve(server: Arc<hyper::Server<impl hyper::service::Service<hyper::Request<hyper::Body>, Response = hyper::Response<hyper::Body>, Error = hyper::Error> + Clone + Send + 'static>>, flags: FlagConfig, logger: slog::Logger) -> Result<(), Box<dyn std::error::Error>> {
    if flags.web_systemd_socket && flags.web_listen_addresses.is_empty() {
        return Err("no web listen address or systemd socket flag specified".into());
    }

    if flags.web_systemd_socket {
        logger.info("Listening on systemd activated listeners instead of port listeners.");
        let listeners = activation::listening_fds()?;
        if listeners.is_empty() {
            return Err("no socket activation file descriptors found".into());
        }
        return serve_multiple(listeners, server, flags, logger).await;
    }

    let mut listeners = Vec::new();
    for address in flags.web_listen_addresses {
        let listener = TcpListener::bind(address)?;
        listeners.push(TokioTcpListener::from_std(listener)?);
    }

    serve_multiple(listeners, server, flags, logger).await
}

pub async fn serve(listener: TokioTcpListener, server: Arc<hyper::Server<impl hyper::service::Service<hyper::Request<hyper::Body>, Response = hyper::Response<hyper::Body>, Error = hyper::Error> + Clone + Send + 'static>>, flags: FlagConfig, logger: slog::Logger) -> Result<(), Box<dyn std::error::Error>> {
    logger.info("Listening on", slog::o!("address" => listener.local_addr().unwrap().to_string()));
    let tls_config_path = &flags.web_config_file;
    if tls_config_path.is_empty() {
        logger.info("TLS is disabled.", slog::o!("http2" => false, "address" => listener.local_addr().unwrap().to_string()));
        return server.serve(TcpListenerStream::new(listener)).await;
    }

    validate_users(tls_config_path).await?;

    let config = get_config(tls_config_path).await?;
    let tls_config = config_to_tls_config(&config.tls_config).await?;

    let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));
    let tls_server = server.with_graceful_shutdown(async {
        tokio::signal::ctrl_c().await.expect("failed to install CTRL+C signal handler");
    });

    let mut incoming = TcpListenerStream::new(listener);
    while let Some(stream) = incoming.next().await {
        match stream {
            Ok(stream) => {
                let tls_acceptor = tls_acceptor.clone();
                let tls_server = tls_server.clone();
                tokio::spawn(async move {
                    match tls_acceptor.accept(stream).await {
                        Ok(tls_stream) => {
                            if let Err(err) = tls_server.serve_connection(tls_stream, hyper::service::service_fn(|req| async move {
                                Ok::<_, hyper::Error>(hyper::Response::new(hyper::Body::from("Hello World")))
                            })).await {
                                logger.error("Error serving TLS connection", slog::o!("error" => err.to_string()));
                            }
                        }
                        Err(err) => {
                            logger.error("Error accepting TLS connection", slog::o!("error" => err.to_string()));
                        }
                    }
                });
            }
            Err(err) => {
                logger.error("Error accepting connection", slog::o!("error" => err.to_string()));
            }
        }
    }

    Ok(())
}

pub async fn validate(tls_config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if tls_config_path.is_empty() {
        return Ok(());
    }
    validate_users(tls_config_path).await?;
    let config = get_config(tls_config_path).await?;
    config_to_tls_config(&config.tls_config).await?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct Cipher(u16);

impl<'de> serde::Deserialize<'de> for Cipher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        for cs in rustls::ALL_CIPHERSUITES.iter() {
            if cs.suite().to_string() == s {
                return Ok(Cipher(cs.suite().get_u16()));
            }
        }
        Err(serde::de::Error::custom(format!("unknown cipher: {}", s)))
    }
}

impl serde::Serialize for Cipher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        for cs in rustls::ALL_CIPHERSUITES.iter() {
            if cs.suite().get_u16() == self.0 {
                return serializer.serialize_str(&cs.suite().to_string());
            }
        }
        Err(serde::ser::Error::custom(format!("unknown cipher: {}", self.0)))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Curve(rustls::NamedGroup);

impl<'de> serde::Deserialize<'de> for Curve {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "CurveP256" => Ok(Curve(rustls::NamedGroup::secp256r1)),
            "CurveP384" => Ok(Curve(rustls::NamedGroup::secp384r1)),
            "CurveP521" => Ok(Curve(rustls::NamedGroup::secp521r1)),
            "X25519" => Ok(Curve(rustls::NamedGroup::x25519)),
            _ => Err(serde::de::Error::custom(format!("unknown curve: {}", s))),
        }
    }
}

impl serde::Serialize for Curve {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self.0 {
            rustls::NamedGroup::secp256r1 => "CurveP256",
            rustls::NamedGroup::secp384r1 => "CurveP384",
            rustls::NamedGroup::secp521r1 => "CurveP521",
            rustls::NamedGroup::x25519 => "X25519",
            _ => return Err(serde::ser::Error::custom(format!("unknown curve: {:?}", self.0))),
        };
        serializer.serialize_str(s)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TLSVersion(u16);

impl<'de> serde::Deserialize<'de> for TLSVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "TLS13" => Ok(TLSVersion(rustls::ProtocolVersion::TLSv1_3.get_u16())),
            "TLS12" => Ok(TLSVersion(rustls::ProtocolVersion::TLSv1_2.get_u16())),
            "TLS11" => Ok(TLSVersion(rustls::ProtocolVersion::TLSv1_1.get_u16())),
            "TLS10" => Ok(TLSVersion(rustls::ProtocolVersion::TLSv1_0.get_u16())),
            _ => Err(serde::de::Error::custom(format!("unknown TLS version: {}", s))),
        }
    }
}

impl serde::Serialize for TLSVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self.0 {
            v if v == rustls::ProtocolVersion::TLSv1_3.get_u16() => "TLS13",
            v if v == rustls::ProtocolVersion::TLSv1_2.get_u16() => "TLS12",
            v if v == rustls::ProtocolVersion::TLSv1_1.get_u16() => "TLS11",
            v if v == rustls::ProtocolVersion::TLSv1_0.get_u16() => "TLS10",
            _ => return Err(serde::ser::Error::custom(format!("unknown TLS version: {}", self.0))),
        };
        serializer.serialize_str(s)
    }
}