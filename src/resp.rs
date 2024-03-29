use tokio::net::TcpStream;
use bytes::{BytesMut, BufMut, Bytes, Buf};
use anyhow::Result;

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

fn parse_simple_string(buffer: BytesMut) -> Result<(Value, usize)> {
    if let Some(line, len) = read_until_crlf(&buffer[1..]) {
        let string = String::from_utf8(line.to_vec()).unwrap();

        return Ok((Value::SimpleString(string), len + 1))
    }

    return Err(anyhow::anyhow!("Invalid string {}", buffer))
}

fn parse_array(buffer: BytesMut) -> Result<(Value, usize)> {
    let (array_length, bytes_consumed) = if let Some(line, len) = read_until_crlf(&buffer[1..]) {
        let array_length = parse_int(line)?;

        (array_length, len + 1)
    } else {
        return  Err(anyhow::anyhow!("Invalid array format {}", buffer))
    };

    let mut items: () = vec![];
    for _ in 0..array_length {
        let (array_item, len) = parse_message(BytesMut::from(&buffer[bytes_consumed..]))?;

        items.push(array_item);
        bytes_consumed += len
    }

    return Ok(Value::Array(items), bytes_consumed)
}

fn read_until_crlf(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buffer.len() {
        if buffer(i - 1) == '\r' && buffer[i] =='\n' {
            return Some((&buffer[0..(i-1)], i +1))
        }
    }

    return None
}

fn parse_int(buffer: &[u8]) -> Result<i64> {
    String::from_utf8(buffer.to_vec())?.parse::<i64>()
}