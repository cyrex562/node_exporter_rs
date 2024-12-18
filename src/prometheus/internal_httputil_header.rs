use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
enum OctetType {
    IsToken = 1 << 0,
    IsSpace = 1 << 1,
}

impl OctetType {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(OctetType::IsToken),
            2 => Some(OctetType::IsSpace),
            _ => None,
        }
    }
}

static mut OCTET_TYPES: [u8; 256] = [0; 256];

fn init_octet_types() {
    unsafe {
        for c in 0..256 {
            let mut t = 0;
            let is_ctl = c <= 31 || c == 127;
            let is_char = c <= 127;
            let is_separator = b" \t\"(),/:;<=>?@[\\]{}".contains(&(c as u8));
            if b" \t\r\n".contains(&(c as u8)) {
                t |= OctetType::IsSpace as u8;
            }
            if is_char && !is_ctl && !is_separator {
                t |= OctetType::IsToken as u8;
            }
            OCTET_TYPES[c] = t;
        }
    }
}

#[derive(Debug)]
struct AcceptSpec {
    value: String,
    q: f64,
}

fn parse_accept(headers: &HashMap<String, String>, key: &str) -> Vec<AcceptSpec> {
    let mut specs = Vec::new();
    if let Some(header_value) = headers.get(key) {
        for s in header_value.split(',') {
            let mut s = s.trim();
            loop {
                let (value, rest) = expect_token_slash(s);
                if value.is_empty() {
                    break;
                }
                let mut spec = AcceptSpec { value, q: 1.0 };
                s = skip_space(rest);
                if s.starts_with(';') {
                    s = skip_space(&s[1..]);
                    if !s.starts_with("q=") {
                        break;
                    }
                    let (q, rest) = expect_quality(&s[2..]);
                    if q < 0.0 {
                        break;
                    }
                    spec.q = q;
                    s = rest;
                }
                specs.push(spec);
                s = skip_space(s);
                if !s.starts_with(',') {
                    break;
                }
                s = skip_space(&s[1..]);
            }
        }
    }
    specs
}

fn skip_space(s: &str) -> &str {
    let mut i = 0;
    for (index, &b) in s.as_bytes().iter().enumerate() {
        if unsafe { OCTET_TYPES[b as usize] & OctetType::IsSpace as u8 } == 0 {
            i = index;
            break;
        }
    }
    &s[i..]
}

fn expect_token_slash(s: &str) -> (String, &str) {
    let mut i = 0;
    for (index, &b) in s.as_bytes().iter().enumerate() {
        if (unsafe { OCTET_TYPES[b as usize] & OctetType::IsToken as u8 } == 0) && b != b'/' {
            i = index;
            break;
        }
    }
    (s[..i].to_string(), &s[i..])
}

fn expect_quality(s: &str) -> (f64, &str) {
    if s.is_empty() {
        return (-1.0, "");
    }
    let mut q = match s.chars().next().unwrap() {
        '0' => 0.0,
        '1' => 1.0,
        _ => return (-1.0, ""),
    };
    let mut s = &s[1..];
    if !s.starts_with('.') {
        return (q, s);
    }
    s = &s[1..];
    let mut n = 0;
    let mut d = 1;
    for (index, &b) in s.as_bytes().iter().enumerate() {
        if b < b'0' || b > b'9' {
            s = &s[index..];
            break;
        }
        n = n * 10 + (b - b'0') as i32;
        d *= 10;
    }
    (q + (n as f64 / d as f64), s)
}

// fn main() {
//     init_octet_types();

//     let mut headers = HashMap::new();
//     headers.insert("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string());

//     let specs = parse_accept(&headers, "Accept");
//     for spec in specs {
//         println!("{:?}", spec);
//     }
// }