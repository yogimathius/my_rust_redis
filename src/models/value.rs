use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    SimpleString(String),
    BulkString(Vec<u8>),
    Array(Vec<Value>),
    Hash(HashMap<String, Value>),
    Integer(i64),
    Error(String),
    BulkBytes(Vec<u8>),
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
                    serialized.push_str(&Value::BulkString(key.into()).serialize());
                    serialized.push_str(&value.serialize());
                }
                format!("*{}\r\n{}", hash.len() * 2, serialized)
            }
            Value::SimpleString(s) => format!("+{}\r\n", s),
            Value::BulkString(s) => {
                let mut serialized = format!("${}\r\n", s.len());
                for byte in s {
                    serialized.push_str(&format!("{}", byte));
                }

                serialized
            }
            Value::NullBulkString => format!("$-1\r\n"),
            Value::Integer(i) => format!(":{}\r\n", i),
            Value::Error(e) => format!("-{}\r\n", e),
            Value::BulkBytes(b) => {
                let mut serialized = format!("${}\r\n", b.len());
                for byte in b {
                    serialized.push_str(&format!("{}", byte));
                }

                serialized
            }
        }
    }
}
