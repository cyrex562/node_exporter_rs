use std::collections::HashMap;
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelName(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelValue(String);

pub type LabelSet = HashMap<LabelName, LabelValue>;

impl LabelSet {
    pub fn to_string(&self) -> String {
        let mut label_names: Vec<&LabelName> = self.keys().collect();
        label_names.sort_by(|a, b| a.0.cmp(&b.0));

        let mut result = String::with_capacity(1024);
        result.push('{');
        for (i, name) in label_names.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            write!(result, "{}=\"{}\"", name.0, self.get(name).unwrap().0).unwrap();
        }
        result.push('}');
        result
    }
}