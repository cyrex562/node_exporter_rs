pub fn safe_dereference<T: Default>(s: &[Option<&T>]) -> Vec<T> {
    s.iter()
        .map(|v| v.map_or_else(T::default, |&val| val.clone()))
        .collect()
}