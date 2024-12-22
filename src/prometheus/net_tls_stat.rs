use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Default)]
pub struct TLSStat {
    pub tls_curr_tx_sw: i32,
    pub tls_curr_rx_sw: i32,
    pub tls_curr_tx_device: i32,
    pub tls_curr_rx_device: i32,
    pub tls_tx_sw: i32,
    pub tls_rx_sw: i32,
    pub tls_tx_device: i32,
    pub tls_rx_device: i32,
    pub tls_decrypt_error: i32,
    pub tls_rx_device_resync: i32,
    pub tls_decrypt_retry: i32,
    pub tls_rx_no_pad_violation: i32,
}

#[derive(Debug, Error)]
pub enum TLSStatError {
    #[error("file read error")]
    FileReadError(#[from] io::Error),
    #[error("parse error")]
    ParseError(#[from] std::num::ParseIntError),
}

pub fn new_tls_stat<P: AsRef<Path>>(path: P) -> Result<TLSStat, TLSStatError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    parse_tls_stat(reader)
}

fn parse_tls_stat<R: BufRead>(reader: R) -> Result<TLSStat, TLSStatError> {
    let mut tls_stat = TLSStat::default();
    let mut lines = reader.lines();

    while let Some(line) = lines.next() {
        let line = line?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() != 2 {
            continue;
        }

        let value = fields[1].parse::<i32>()?;
        match fields[0] {
            "TLSCurrTxSw:" => tls_stat.tls_curr_tx_sw = value,
            "TLSCurrRxSw:" => tls_stat.tls_curr_rx_sw = value,
            "TLSCurrTxDevice:" => tls_stat.tls_curr_tx_device = value,
            "TLSCurrRxDevice:" => tls_stat.tls_curr_rx_device = value,
            "TLSTxSw:" => tls_stat.tls_tx_sw = value,
            "TLSRxSw:" => tls_stat.tls_rx_sw = value,
            "TLSTxDevice:" => tls_stat.tls_tx_device = value,
            "TLSRxDevice:" => tls_stat.tls_rx_device = value,
            "TLSDecryptError:" => tls_stat.tls_decrypt_error = value,
            "TLSRxDeviceResync:" => tls_stat.tls_rx_device_resync = value,
            "TLSDecryptRetry:" => tls_stat.tls_decrypt_retry = value,
            "TLSRxNoPadViolation:" => tls_stat.tls_rx_no_pad_violation = value,
            _ => {}
        }
    }

    Ok(tls_stat)
}
