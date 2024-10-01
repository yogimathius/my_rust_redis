use anyhow::Error;
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use crate::{log, models::value::Value, utilities::parse_message};

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(1024),
        }
    }

    pub async fn read_value(&mut self) -> Result<Option<Value>, Error> {
        let bytes_read = self.stream.read_buf(&mut self.buffer).await?;
        log!("bytes_read {:?}", bytes_read);
        if bytes_read == 0 {
            return Ok(None);
        }

        let (v, _) = parse_message(self.buffer.split())?;

        Ok(Some(v))
    }

    pub async fn write_value(&mut self, value: Value) -> Result<(), Error> {
        match value {
            Value::SimpleString(s) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(s.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Value::NullBulkString => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Value::BulkString(s) => {
                self.write_bulk(&s).await?;
            }
            Value::Array(a) => {
                let l = a.len().to_string();
                self.stream.write_u8(b'*').await?;
                self.stream.write_all(l.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
                for s in a {
                    self.write_bulk(&s.clone().serialize()).await?;
                }
            }
            Value::Integer(i) => {
                self.stream.write_u8(b':').await?;
                self.stream.write_all(i.to_string().as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Value::Hash(h) => println!("{:?}", h),
            Value::Error(_) => todo!(),
            Value::BulkBytes(b) => {
                self.write_bytes(&b).await?;
            } // Value::Error => {}
        }

        self.stream.flush().await?;

        Ok(())
    }

    async fn write_bulk(&mut self, s: &str) -> Result<(), Error> {
        let l = s.len().to_string();
        self.stream.write_u8(b'$').await?;
        self.stream.write_all(l.as_bytes()).await?;
        self.stream.write_all(b"\r\n").await?;
        self.stream.write_all(s.as_bytes()).await?;
        self.stream.write_all(b"\r\n").await?;
        Ok(())
    }

    async fn write_bytes(&mut self, b: &[u8]) -> Result<(), Error> {
        let l = b.len().to_string();
        self.stream.write_u8(b'$').await?;
        self.stream.write_all(l.as_bytes()).await?;
        self.stream.write_all(b"\r\n").await?;
        self.stream.write_all(b).await?;
        Ok(())
    }
}
