use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::str::FromStr;

#[derive(Default, Debug)]
struct Crypto {
    alignmask: Option<u64>,
    async_: bool,
    blocksize: Option<u64>,
    chunksize: Option<u64>,
    ctxsize: Option<u64>,
    digestsize: Option<u64>,
    driver: String,
    geniv: String,
    internal: String,
    ivsize: Option<u64>,
    maxauthsize: Option<u64>,
    max_keysize: Option<u64>,
    min_keysize: Option<u64>,
    module: String,
    name: String,
    priority: Option<i64>,
    refcnt: Option<i64>,
    seedsize: Option<u64>,
    selftest: String,
    type_: String,
    walksize: Option<u64>,
}

impl Crypto {
    fn parse_kv(&mut self, k: &str, v: &str) -> Result<(), Box<dyn std::error::Error>> {
        match k {
            "async" => self.async_ = v == "yes",
            "blocksize" => self.blocksize = Some(v.parse()?),
            "chunksize" => self.chunksize = Some(v.parse()?),
            "digestsize" => self.digestsize = Some(v.parse()?),
            "driver" => self.driver = v.to_string(),
            "geniv" => self.geniv = v.to_string(),
            "internal" => self.internal = v.to_string(),
            "ivsize" => self.ivsize = Some(v.parse()?),
            "maxauthsize" => self.maxauthsize = Some(v.parse()?),
            "max keysize" => self.max_keysize = Some(v.parse()?),
            "min keysize" => self.min_keysize = Some(v.parse()?),
            "module" => self.module = v.to_string(),
            "name" => self.name = v.to_string(),
            "priority" => self.priority = Some(v.parse()?),
            "refcnt" => self.refcnt = Some(v.parse()?),
            "seedsize" => self.seedsize = Some(v.parse()?),
            "selftest" => self.selftest = v.to_string(),
            "type" => self.type_ = v.to_string(),
            "walksize" => self.walksize = Some(v.parse()?),
            _ => {}
        }
        Ok(())
    }
}

fn parse_crypto<R: BufRead>(reader: R) -> Result<Vec<Crypto>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    let mut current_crypto = Crypto::default();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("name") {
            if !current_crypto.name.is_empty() {
                out.push(current_crypto);
                current_crypto = Crypto::default();
            }
        } else if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, ":").collect();
        if parts.len() != 2 {
            return Err(format!("Cannot parse line: {}", line).into());
        }

        let k = parts[0].trim();
        let v = parts[1].trim();
        current_crypto.parse_kv(k, v)?;
    }

    if !current_crypto.name.is_empty() {
        out.push(current_crypto);
    }

    Ok(out)
}

fn read_crypto_file<P: AsRef<Path>>(path: P) -> Result<Vec<Crypto>, Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    parse_crypto(reader)
}