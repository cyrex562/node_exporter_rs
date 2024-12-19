const OFFSET64: u64 = 14695981039346656037;
const PRIME64: u64 = 1099511628211;

pub fn hash_new() -> u64 {
    OFFSET64
}

pub fn hash_add(mut h: u64, s: &str) -> u64 {
    for byte in s.bytes() {
        h ^= byte as u64;
        h = h.wrapping_mul(PRIME64);
    }
    h
}

pub fn hash_add_byte(mut h: u64, b: u8) -> u64 {
    h ^= b as u64;
    h = h.wrapping_mul(PRIME64);
    h
}