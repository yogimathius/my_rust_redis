use anyhow::Error;
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{broadcast, Mutex},
};

use crate::{log, models::value::Value, utilities::parse_message};
use std::sync::Arc;

#[derive(Debug)]
pub struct Connection {
    pub stream: TcpStream,
    buffer: BytesMut,
}

impl Connection {
    pub async fn new(replicaof: Option<String>, stream: Option<TcpStream>) -> Self {
        if let Some(stream) = stream {
            Connection {
                stream,
                buffer: BytesMut::with_capacity(1024),
            }
        } else {
            let stream = TcpStream::connect(replicaof.unwrap()).await.unwrap();
            Connection {
                stream,
                buffer: BytesMut::with_capacity(1024),
            }
        }
    }

    pub async fn read_value(&mut self) -> Result<Option<Value>, Error> {
        let bytes_read = self.stream.read_buf(&mut self.buffer).await?;
        log!("bytes_read {:?}", bytes_read);
        if bytes_read == 0 {
            return Ok(None);
        }

        let (v, _) = parse_message(self.buffer.split())?;
        log!("Read value {:?}", v);
        Ok(Some(v))
    }

    pub async fn expect_read(&mut self, expected: &str) {
        match self.stream.read_buf(&mut self.buffer).await {
            Ok(bytes_read) => {
                let response = std::str::from_utf8(&self.buffer[..bytes_read]).unwrap();
                let trimmed = response.trim();
                if trimmed != expected {
                    panic!(
                        "Unexpected response from master: {} (expected {})",
                        trimmed, expected
                    );
                }
            }
            Err(e) => {
                panic!("Error reading from master: {}", e);
            }
        }
    }

    pub async fn write_value(&mut self, value: Value) -> Result<(), Error> {
        log!("Writing value {:?}", value.clone().serialize().as_bytes());
        {
            self.stream.write_all(value.serialize().as_bytes()).await?;
            self.stream.flush().await?;
        }
        Ok(())
    }

    pub async fn write_bulk(&mut self, s: &str) -> Result<(), Error> {
        log!("Writing bulk {:?}", s);
        let l = s.len().to_string();
        {
            self.stream.write_u8(b'$').await?;
            self.stream.write_all(l.as_bytes()).await?;
            self.stream.write_all(b"\r\n").await?;
            self.stream.write_all(s.as_bytes()).await?;
            self.stream.write_all(b"\r\n").await?;
        }
        Ok(())
    }

    pub async fn write_all(&mut self, b: &[u8]) -> Result<(), Error> {
        self.stream.write_all(b).await?;
        Ok(())
    }

    pub fn spawn_pubsub_task(&mut self, receiver: Arc<Mutex<broadcast::Receiver<Value>>>) {
        let mut conn = self.clone();
        tokio::spawn(async move {
            while let Ok(f) = receiver.lock().await.recv().await {
                log!("Sending value: {:?}", f);
                if let Err(e) = conn.write_value(f).await {
                    log!("Failed to send value: {:?}", e);
                    break;
                }
            }
            log!("Spawning pubsub task done");
        });
    }
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        Connection {
            stream: self.stream,
            buffer: BytesMut::with_capacity(1024),
        }
    }
}
