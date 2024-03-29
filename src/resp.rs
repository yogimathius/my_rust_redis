use tokio::net::TcpStream;
use bytes::{BytesMut, BufMut, Bytes, Buf};

pub enum Value {
    SimpleString(String),
    BulkString(String),
    Array(Vec<Value>)
}

impl Value {
    pub fn serialize(self) -> String {
        match self {
            Value::SimpleString(s) => format!("+{}\r\n", s),
            Value::BulkString(s) => format!("${}\r\n{}\r\n", s.chars().count(), s),
            _ => panic!("Not implemented")
        }
    }
}
pub struct RespHandler {
    stream: TcpStream,
    buffer: ByteMut
}

impl RespHandler {
    pub fn new(stream: TcpStream) -> Self {
        RespHandler {
            stream,
            buffer: BytesMut::with_capacity(512)
        }
    }

    pub async fn read_value(&mut self) -> Result<Value> {

    }

    pub async fn write_value(&mut self, value: Value) -> Result<()> {

    }
}

fn parse_message(buffer: BytesMut) -> Result<(Value, usize)> {
    match buffer[0] as char {
        '+' => parse_simple_string(buffer),
        '*' => parse_array(buffer),
        '$' => parse_bulk_string(buffer),
        _ => Err(anyhow::anyhow!("Unknown value type {}", buffer))
    }
}

fn read_until_crlf(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buffer.len() {
        if buffer(i - 1) == '\r' && buffer[i] =='\n' {
            return Some((&buffer[0..(i-1)], i +1))
        }
    }

    return None
}