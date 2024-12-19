use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

pub trait Value {
    fn value_type(&self) -> ValueType;
    fn to_string(&self) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    None,
    Scalar,
    Vector,
    Matrix,
    String,
}

impl Serialize for ValueType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ValueType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "<ValNone>" => Ok(ValueType::None),
            "scalar" => Ok(ValueType::Scalar),
            "vector" => Ok(ValueType::Vector),
            "matrix" => Ok(ValueType::Matrix),
            "string" => Ok(ValueType::String),
            _ => Err(serde::de::Error::custom(format!("unknown value type {}", s))),
        }
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ValueType::None => "<ValNone>",
            ValueType::Scalar => "scalar",
            ValueType::Vector => "vector",
            ValueType::Matrix => "matrix",
            ValueType::String => "string",
        };
        write!(f, "{}", s)
    }
}

// Example implementations for the Value trait
pub struct Matrix;
pub struct Vector;
pub struct Scalar;
pub struct StringValue;

impl Value for Matrix {
    fn value_type(&self) -> ValueType {
        ValueType::Matrix
    }

    fn to_string(&self) -> String {
        "Matrix".to_string()
    }
}

impl Value for Vector {
    fn value_type(&self) -> ValueType {
        ValueType::Vector
    }

    fn to_string(&self) -> String {
        "Vector".to_string()
    }
}

impl Value for Scalar {
    fn value_type(&self) -> ValueType {
        ValueType::Scalar
    }

    fn to_string(&self) -> String {
        "Scalar".to_string()
    }
}

impl Value for StringValue {
    fn value_type(&self) -> ValueType {
        ValueType::String
    }

    fn to_string(&self) -> String {
        "String".to_string()
    }
}