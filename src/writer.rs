use anyhow::Error;
use bytes::BytesMut;

use crate::{log, models::value::Value, utilities::parse_message};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

#[derive(Debug)]
pub struct Writer {
    buffer: BytesMut,
}

impl Writer {
    pub fn new() -> Self {
        Writer {
            buffer: BytesMut::with_capacity(1024),
        }
    }

    pub async fn read_value(&mut self, stream: &mut TcpStream) -> Result<Option<Value>, Error> {
        let bytes_read = stream.read(&mut self.buffer);
        log!("bytes_read {:?}", bytes_read);
        if let Ok(0) = bytes_read {
            return Ok(None);
        }

        let (v, _) = parse_message(self.buffer.split())?;
        log!("Read value {:?}", v);
        Ok(Some(v))
    }

    pub async fn expect_read(&mut self, expected: &str, stream: &mut TcpStream) {
        {
            let response = self.read_value(stream).await.unwrap();
            log!("Read value {:?}", response);
            if let Some(Value::SimpleString(s)) = response {
                if s != expected {
                    panic!(
                        "Unexpected response from master: {} (expected {})",
                        s, expected
                    );
                }
            } else {
                panic!("Unexpected response from master: {:?}", response);
            }
        }

        // match self.stream.read_buf(&mut self.buffer).await {
        //     Ok(bytes_read) => {
        //         log!("bytes_read {:?}", bytes_read);
        //         let response = std::str::from_utf8(&self.buffer[..bytes_read]).unwrap();
        //         let trimmed = response.trim();
        //         if trimmed != expected {
        //             panic!(
        //                 "Unexpected response from master: {} (expected {})",
        //                 trimmed, expected
        //             );
        //         }
        //     }
        //     Err(e) => {
        //         panic!("Error reading from master: {}", e);
        //     }
        // }
    }

    pub async fn write_value(&mut self, value: Value, stream: &mut TcpStream) -> Result<(), Error> {
        log!("Writing value {:?}", value.clone().serialize().as_bytes());
        {
            let _ = stream.write_all(value.serialize().as_bytes());
            let _ = stream.flush();
        }
        Ok(())
    }

    pub async fn write_bulk(&mut self, s: &str, mut stream: TcpStream) -> Result<(), Error> {
        log!("Writing bulk {:?}", s);
        let l = s.len().to_string();
        {
            let _ = stream.write_all(b"$");
            let _ = stream.write_all(l.as_bytes());
            let _ = stream.write_all(b"\r\n");
            let _ = stream.write_all(s.as_bytes());
            let _ = stream.write_all(b"\r\n");
        }
        Ok(())
    }

    pub async fn write_all(&mut self, b: &[u8], stream: &mut TcpStream) -> Result<(), Error> {
        let _ = stream.write_all(b);
        Ok(())
    }
}
