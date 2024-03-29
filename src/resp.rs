use tokio::net::TcpStream;
use bytes::{BytesMut, BufMut, Bytes, Buf};

pub enum Value {

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