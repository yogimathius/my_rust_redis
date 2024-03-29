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