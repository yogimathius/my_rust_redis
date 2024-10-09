use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    SimpleString(String),
    BulkString(String),
    Array(Vec<Value>),
    Hash(HashMap<String, Value>),
    Integer(i64),
    Error(String),
    NullBulkString,
}

impl Value {
    pub fn serialize(self) -> String {
        match self {
            Value::Array(values) => {
                let mut serialized = format!("*{}\r\n", values.len());
                for value in values {
                    serialized.push_str(&value.serialize());
                }

                serialized
            }
            Value::Hash(hash) => {
                let mut serialized = String::new();
                for (key, value) in hash.clone() {
                    serialized.push_str(&Value::BulkString(key).serialize());
                    serialized.push_str(&value.serialize());
                }
                format!("*{}\r\n{}", hash.len() * 2, serialized)
            }
            Value::SimpleString(s) => format!("+{}\r\n", s),
            Value::BulkString(s) => format!("${}\r\n{}\r\n", s.chars().count(), s),
            Value::NullBulkString => format!("$-1\r\n"),
            Value::Integer(i) => format!(":{}\r\n", i),
            Value::Error(e) => format!("-{}\r\n", e),
        }
    }
}
