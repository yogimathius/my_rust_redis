use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    SimpleString(String),
    BulkString(String),
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
            Value::BulkBytes(b) => {
                let mut serialized = format!("${}\r\n", b.len());
                for byte in b {
                    serialized.push_str(&format!("{}", byte));
                }

                serialized
            }
        }
    }

    pub fn serialize_to_binary(&self) -> Result<Vec<u8>, String> {
        let mut buffer = Vec::new();

        match self {
            Value::SimpleString(s) => {
                buffer.push(0x01); // Type byte for SimpleString
                buffer.extend_from_slice(&(s.len() as u32).to_be_bytes());
                buffer.extend_from_slice(s.as_bytes());
            }
            Value::BulkString(s) => {
                buffer.push(0x02); // Type byte for BulkString
                buffer.extend_from_slice(&(s.len() as u32).to_be_bytes());
                buffer.extend_from_slice(s.as_bytes());
            }
            Value::Array(values) => {
                buffer.push(0x03); // Type byte for Array
                buffer.extend_from_slice(&(values.len() as u32).to_be_bytes());
                for value in values {
                    buffer.extend_from_slice(&value.serialize_to_binary()?);
                }
            }
            Value::Hash(hash) => {
                buffer.push(0x04); // Type byte for Hash
                buffer.extend_from_slice(&(hash.len() as u32).to_be_bytes());
                for (key, value) in hash {
                    buffer
                        .extend_from_slice(&Value::BulkString(key.clone()).serialize_to_binary()?);
                    buffer.extend_from_slice(&value.serialize_to_binary()?);
                }
            }
            Value::Integer(i) => {
                buffer.push(0x05); // Type byte for Integer
                buffer.extend_from_slice(&i.to_be_bytes());
            }
            Value::Error(e) => {
                buffer.push(0x06); // Type byte for Error
                buffer.extend_from_slice(&(e.len() as u32).to_be_bytes());
                buffer.extend_from_slice(e.as_bytes());
            }
            Value::BulkBytes(b) => {
                buffer.push(0x07); // Type byte for BulkBytes
                buffer.extend_from_slice(&(b.len() as u32).to_be_bytes());
                buffer.extend_from_slice(b);
            }
            Value::NullBulkString => {
                buffer.push(0x08); // Type byte for NullBulkString
            }
        }

        Ok(buffer)
    }
}
