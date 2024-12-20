use regex::Regex;

struct DeviceFilter {
    ignore_pattern: Option<Regex>,
    accept_pattern: Option<Regex>,
}

impl DeviceFilter {
    fn new(ignored_pattern: &str, accept_pattern: &str) -> Self {
        let ignore_pattern = if !ignored_pattern.is_empty() {
            Some(Regex::new(ignored_pattern).unwrap())
        } else {
            None
        };

        let accept_pattern = if !accept_pattern.is_empty() {
            Some(Regex::new(accept_pattern).unwrap())
        } else {
            None
        };

        DeviceFilter {
            ignore_pattern,
            accept_pattern,
        }
    }

    fn ignored(&self, name: &str) -> bool {
        (self.ignore_pattern.as_ref().map_or(false, |p| p.is_match(name)))
            || (self.accept_pattern.as_ref().map_or(false, |p| !p.is_match(name)))
    }
}