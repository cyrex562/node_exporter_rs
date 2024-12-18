use std::num::ParseIntError;

#[derive(Debug)]
pub struct ValueParser {
    v: String,
    err: Option<ParseIntError>,
}

impl ValueParser {
    // Creates a ValueParser using the input string.
    pub fn new(v: String) -> Self {
        ValueParser { v, err: None }
    }

    // Interprets the underlying value as an i32 and returns that value.
    pub fn int(&mut self) -> i32 {
        self.int64() as i32
    }

    // Interprets the underlying value as an i64 and returns a pointer to that value.
    pub fn pint64(&mut self) -> Option<Box<i64>> {
        if self.err.is_some() {
            return None;
        }

        let v = self.int64();
        Some(Box::new(v))
    }

    // Interprets the underlying value as an i64 and returns that value.
    fn int64(&mut self) -> i64 {
        if let Some(err) = &self.err {
            return 0;
        }

        // A base value of zero makes from_str_radix infer the correct base using the string's prefix, if any.
        match i64::from_str_radix(&self.v, 0) {
            Ok(v) => v,
            Err(err) => {
                self.err = Some(err);
                0
            }
        }
    }

    // Interprets the underlying value as a u64 and returns a pointer to that value.
    pub fn puint64(&mut self) -> Option<Box<u64>> {
        if self.err.is_some() {
            return None;
        }

        // A base value of zero makes from_str_radix infer the correct base using the string's prefix, if any.
        match u64::from_str_radix(&self.v, 0) {
            Ok(v) => Some(Box::new(v)),
            Err(err) => {
                self.err = Some(err);
                None
            }
        }
    }

    // Returns the last error, if any, encountered by the ValueParser.
    pub fn err(&self) -> Option<&ParseIntError> {
        self.err.as_ref()
    }
}

// fn main() {
//     let mut vp = ValueParser::new("123".to_string());

//     let int_value = vp.int();
//     println!("Int value: {}", int_value);

//     if let Some(pint64_value) = vp.pint64() {
//         println!("PInt64 value: {}", pint64_value);
//     }

//     if let Some(puint64_value) = vp.puint64() {
//         println!("PUInt64 value: {}", puint64_value);
//     }

//     if let Some(err) = vp.err() {
//         println!("Error: {}", err);
//     }
// }