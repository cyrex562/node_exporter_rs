use std::collections::HashMap;

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
        if !b.is_ascii_whitespace() {
            i = index;
            break;
        }
    }
    &s[i..]
}

fn expect_token_slash(s: &str) -> (String, &str) {
    let mut i = 0;
    for (index, &b) in s.as_bytes().iter().enumerate() {
        if !b.is_ascii_alphanumeric() && b != b'/' {
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
        if !b.is_ascii_digit() {
            s = &s[index..];
            break;
        }
        n = n * 10 + (b - b'0') as i32;
        d *= 10;
    }
    (q + (n as f64 / d as f64), s)
}

fn negotiate_content_encoding(headers: &HashMap<String, String>, offers: &[&str]) -> String {
    let mut best_offer = "identity".to_string();
    let mut best_q = -1.0;
    let specs = parse_accept(headers, "Accept-Encoding");
    for &offer in offers {
        for spec in &specs {
            if spec.q > best_q && (spec.value == "*" || spec.value == offer) {
                best_q = spec.q;
                best_offer = offer.to_string();
            }
        }
    }
    if best_q == 0.0 {
        best_offer = "".to_string();
    }
    best_offer
}

// fn main() {
//     let mut headers = HashMap::new();
//     headers.insert("Accept-Encoding".to_string(), "gzip, deflate, br;q=0.8, *;q=0.5".to_string());

//     let offers = vec!["gzip", "deflate", "br"];
//     let best_encoding = negotiate_content_encoding(&headers, &offers);
//     println!("Best encoding: {}", best_encoding);
// }