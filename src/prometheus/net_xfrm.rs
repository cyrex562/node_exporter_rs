use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::num::ParseIntError;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Default)]
pub struct XfrmStat {
    pub xfrm_in_error: i32,
    pub xfrm_in_buffer_error: i32,
    pub xfrm_in_hdr_error: i32,
    pub xfrm_in_no_states: i32,
    pub xfrm_in_state_proto_error: i32,
    pub xfrm_in_state_mode_error: i32,
    pub xfrm_in_state_seq_error: i32,
    pub xfrm_in_state_expired: i32,
    pub xfrm_in_state_mismatch: i32,
    pub xfrm_in_state_invalid: i32,
    pub xfrm_in_tmpl_mismatch: i32,
    pub xfrm_in_no_pols: i32,
    pub xfrm_in_pol_block: i32,
    pub xfrm_in_pol_error: i32,
    pub xfrm_out_error: i32,
    pub xfrm_out_bundle_gen_error: i32,
    pub xfrm_out_bundle_check_error: i32,
    pub xfrm_out_no_states: i32,
    pub xfrm_out_state_proto_error: i32,
    pub xfrm_out_state_mode_error: i32,
    pub xfrm_out_state_seq_error: i32,
    pub xfrm_out_state_expired: i32,
    pub xfrm_out_pol_block: i32,
    pub xfrm_out_pol_dead: i32,
    pub xfrm_out_pol_error: i32,
    pub xfrm_fwd_hdr_error: i32,
    pub xfrm_out_state_invalid: i32,
    pub xfrm_acquire_error: i32,
}

#[derive(Debug, Error)]
pub enum XfrmStatError {
    #[error("file read error")]
    FileReadError(#[from] io::Error),
    #[error("parse error")]
    ParseError(#[from] ParseIntError),
    #[error("invalid format: {0}")]
    InvalidFormat(String),
}

pub fn new_xfrm_stat<P: AsRef<Path>>(path: P) -> Result<XfrmStat, XfrmStatError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    parse_xfrm_stat(reader)
}

fn parse_xfrm_stat<R: BufRead>(reader: R) -> Result<XfrmStat, XfrmStatError> {
    let mut xfrm_stat = XfrmStat::default();
    let mut lines = reader.lines();

    while let Some(line) = lines.next() {
        let line = line?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() != 2 {
            return Err(XfrmStatError::InvalidFormat(format!("expected 2 fields, got {}: {}", fields.len(), line)));
        }

        let name = fields[0];
        let value = fields[1].parse::<i32>()?;

        match name {
            "XfrmInError" => xfrm_stat.xfrm_in_error = value,
            "XfrmInBufferError" => xfrm_stat.xfrm_in_buffer_error = value,
            "XfrmInHdrError" => xfrm_stat.xfrm_in_hdr_error = value,
            "XfrmInNoStates" => xfrm_stat.xfrm_in_no_states = value,
            "XfrmInStateProtoError" => xfrm_stat.xfrm_in_state_proto_error = value,
            "XfrmInStateModeError" => xfrm_stat.xfrm_in_state_mode_error = value,
            "XfrmInStateSeqError" => xfrm_stat.xfrm_in_state_seq_error = value,
            "XfrmInStateExpired" => xfrm_stat.xfrm_in_state_expired = value,
            "XfrmInStateMismatch" => xfrm_stat.xfrm_in_state_mismatch = value,
            "XfrmInStateInvalid" => xfrm_stat.xfrm_in_state_invalid = value,
            "XfrmInTmplMismatch" => xfrm_stat.xfrm_in_tmpl_mismatch = value,
            "XfrmInNoPols" => xfrm_stat.xfrm_in_no_pols = value,
            "XfrmInPolBlock" => xfrm_stat.xfrm_in_pol_block = value,
            "XfrmInPolError" => xfrm_stat.xfrm_in_pol_error = value,
            "XfrmOutError" => xfrm_stat.xfrm_out_error = value,
            "XfrmOutBundleGenError" => xfrm_stat.xfrm_out_bundle_gen_error = value,
            "XfrmOutBundleCheckError" => xfrm_stat.xfrm_out_bundle_check_error = value,
            "XfrmOutNoStates" => xfrm_stat.xfrm_out_no_states = value,
            "XfrmOutStateProtoError" => xfrm_stat.xfrm_out_state_proto_error = value,
            "XfrmOutStateModeError" => xfrm_stat.xfrm_out_state_mode_error = value,
            "XfrmOutStateSeqError" => xfrm_stat.xfrm_out_state_seq_error = value,
            "XfrmOutStateExpired" => xfrm_stat.xfrm_out_state_expired = value,
            "XfrmOutPolBlock" => xfrm_stat.xfrm_out_pol_block = value,
            "XfrmOutPolDead" => xfrm_stat.xfrm_out_pol_dead = value,
            "XfrmOutPolError" => xfrm_stat.xfrm_out_pol_error = value,
            "XfrmFwdHdrError" =