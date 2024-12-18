use std::fs;
use std::num::ParseIntError;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    ParseIntError(ParseIntError),
    InvalidFormat,
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> ParseError {
        ParseError::IoError(err)
    }
}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> ParseError {
        ParseError::ParseIntError(err)
    }
}

pub fn parse_uint32s(ss: &[String]) -> Result<Vec<u32>, ParseError> {
    ss.iter()
        .map(|s| u32::from_str(s).map_err(ParseError::from))
        .collect()
}

pub fn parse_uint64s(ss: &[String]) -> Result<Vec<u64>, ParseError> {
    ss.iter()
        .map(|s| u64::from_str(s).map_err(ParseError::from))
        .collect()
}

pub fn parse_pint64s(ss: &[String]) -> Result<Vec<Box<i64>>, ParseError> {
    ss.iter()
        .map(|s| i64::from_str(s).map(|i| Box::new(i)).map_err(ParseError::from))
        .collect()
}

pub fn parse_hex_uint64s(ss: &[String]) -> Result<Vec<Box<u64>>, ParseError> {
    ss.iter()
        .map(|s| u64::from_str_radix(s, 16).map(|u| Box::new(u)).map_err(ParseError::from))
        .collect()
}

pub fn read_uint_from_file<P: AsRef<Path>>(path: P) -> Result<u64, ParseError> {
    let data = fs::read_to_string(path)?;
    u64::from_str(data.trim()).map_err(ParseError::from)
}

pub fn read_int_from_file<P: AsRef<Path>>(path: P) -> Result<i64, ParseError> {
    let data = fs::read_to_string(path)?;
    i64::from_str(data.trim()).map_err(ParseError::from)
}

pub fn parse_bool(b: &str) -> Option<bool> {
    match b {
        "enabled" => Some(true),
        "disabled" => Some(false),
        _ => None,
    }
}

pub fn read_hex_from_file<P: AsRef<Path>>(path: P) -> Result<u64, ParseError> {
    let data = fs::read_to_string(path)?;
    let hex_string = data.trim();
    if !hex_string.starts_with("0x") {
        return Err(ParseError::InvalidFormat);
    }
    u64::from_str_radix(&hex_string[2..], 16).map_err(ParseError::from)
}

// fn main() {
//     // Example usage
//     let strings = vec!["123".to_string(), "456".to_string(), "789".to_string()];
//     match parse_uint32s(&strings) {
//         Ok(nums) => println!("Parsed uint32s: {:?}", nums),
//         Err(e) => println!("Error parsing uint32s: {:?}", e),
//     }

//     match read_uint_from_file("path/to/file") {
//         Ok(num) => println!("Read uint from file: {}", num),
//         Err(e) => println!("Error reading uint from file: {:?}", e),
//     }
// }