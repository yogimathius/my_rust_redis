use clap::Parser as ClapParser;

#[derive(ClapParser, Debug, Clone)]
pub struct Args {
    #[arg(short, long, default_value_t = 6379)]
    pub port: u16,
    #[arg(short, long, value_delimiter = ' ', num_args = 1)]
    pub replicaof: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    SimpleString(String),
    BulkString(String),
    Array(Vec<Value>),
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
            Value::SimpleString(s) => format!("+{}\r\n", s),
            Value::BulkString(s) => format!("${}\r\n{}\r\n", s.chars().count(), s),
            Value::NullBulkString => format!("$-1\r\n"),
        }
    }
}
