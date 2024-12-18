use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DistinguishedName, DnType, IsCa, KeyPair,
    RcgenError, SanType,
};
use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;
use std::time::{Duration, SystemTime};

const VALIDITY_PERIOD: Duration = Duration::from_secs(50 * 365 * 24 * 60 * 60); // 50 years

fn encode_certificate(path: &str, cert: &Certificate) -> Result<(), RcgenError> {
    let pem = cert.serialize_pem()?;
    fs::write(format!("{}.crt", path), pem).unwrap();
    Ok(())
}

fn encode_private_key(path: &str, cert: &Certificate) -> Result<(), RcgenError> {
    let pem = cert.serialize_private_key_pem();
    fs::write(format!("{}.key", path), pem).unwrap();
    Ok(())
}

fn generate_ca(
    common_name: &str,
    issuer_cert: Option<&Certificate>,
    issuer_key: Option<&KeyPair>,
) -> Certificate {
    let mut params = CertificateParams::default();
    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(DnType::CommonName, common_name);
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.not_before = SystemTime::now();
    params.not_after = params.not_before + VALIDITY_PERIOD;
    if let (Some(issuer_cert), Some(issuer_key)) = (issuer_cert, issuer_key) {
        params.key_pair = Some(KeyPair::from_der(&issuer_key.serialize_der().unwrap()).unwrap());
        params.serial_number = Some(issuer_cert.get_serial_number());
    }
    Certificate::from_params(params).unwrap()
}

fn generate_certificate(
    ca_cert: &Certificate,
    ca_key: &KeyPair,
    is_server: bool,
    name: &str,
    ip_addresses: Vec<IpAddr>,
) -> Certificate {
    let mut params = CertificateParams::default();
    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(DnType::CommonName, name);
    params.serial_number = Some(ca_cert.get_serial_number());
    params.not_before = SystemTime::now();
    params.not_after = params.not_before + VALIDITY_PERIOD;
    if is_server {
        params.subject_alt_names = ip_addresses
            .into_iter()
            .map(SanType::IpAddress)
            .collect();
        params.extended_key_usages = vec![
            rcgen::ExtendedKeyUsagePurpose::ServerAuth,
            rcgen::ExtendedKeyUsagePurpose::ClientAuth,
        ];
    } else {
        params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
    }
    Certificate::from_params(params).unwrap()
}

fn write_certificate_and_key(path: &str, cert: &Certificate) {
    encode_certificate(path, cert).unwrap();
    encode_private_key(path, cert).unwrap();
}

fn main() {
    println!("Generating root CA");
    let root_cert = generate_ca("Prometheus Root CA", None, None);

    println!("Generating CA");
    let ca_cert = generate_ca("Prometheus TLS CA", Some(&root_cert), root_cert.get_key_pair());

    println!("Generating server certificate");
    let server_cert = generate_certificate(
        &ca_cert,
        ca_cert.get_key_pair(),
        true,
        "localhost",
        vec![
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 0)),
        ],
    );
    write_certificate_and_key("testdata/server", &server_cert);

    println!("Generating client certificate");
    let client_cert = generate_certificate(&ca_cert, ca_cert.get_key_pair(), false, "localhost", vec![]);
    write_certificate_and_key("testdata/client", &client_cert);

    println!("Generating self-signed client certificate");
    let self_signed_cert = generate_certificate(&root_cert, root_cert.get_key_pair(), false, "localhost", vec![]);
    write_certificate_and_key("testdata/self-signed-client", &self_signed_cert);

    println!("Generating CA bundle");
    let ca_pem = ca_cert.serialize_pem().unwrap();
    let root_pem = root_cert.serialize_pem().unwrap();
    fs::write("testdata/tls-ca-chain.pem", format!("{}\n{}", ca_pem, root_pem)).unwrap();
}