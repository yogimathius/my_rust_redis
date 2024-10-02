use anyhow::Error;
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
    sync::{broadcast, Mutex},
};

use crate::{log, models::value::Value, utilities::parse_message};
use std::sync::Arc;

pub struct Connection {
    stream: Arc<Mutex<BufWriter<TcpStream>>>,
    buffer: BytesMut,
}

impl Connection {
    pub async fn new(replicaof: Option<String>, stream: Option<TcpStream>) -> Self {
        if let Some(stream) = stream {
            Connection {
                stream: Arc::new(Mutex::new(BufWriter::new(stream))),
                buffer: BytesMut::with_capacity(1024),
            }
        } else {
            let stream = TcpStream::connect(replicaof.unwrap()).await.unwrap();
            Connection {
                stream: Arc::new(Mutex::new(BufWriter::new(stream))),
                buffer: BytesMut::with_capacity(1024),
            }
        }
    }

    pub async fn read_value(&mut self) -> Result<Option<Value>, Error> {
        let bytes_read = self.stream.lock().await.read_buf(&mut self.buffer).await?;
        log!("bytes_read {:?}", bytes_read);
        if bytes_read == 0 {
            return Ok(None);
        }

        let (v, _) = parse_message(self.buffer.split())?;
        log!("Read value {:?}", v);
        Ok(Some(v))
    }

    pub async fn write_value(&mut self, value: Value) -> Result<(), Error> {
        log!("Writing value {:?}", value.clone().serialize().as_bytes());
        {
            let mut stream = self.stream.lock().await;
            stream.write_all(value.serialize().as_bytes()).await?;
            stream.flush().await?;
        }
        Ok(())
    }

    pub async fn write_bulk(&mut self, s: &str) -> Result<(), Error> {
        log!("Writing bulk {:?}", s);
        let l = s.len().to_string();
        {
            let mut stream = self.stream.lock().await;
            stream.write_u8(b'$').await?;
            stream.write_all(l.as_bytes()).await?;
            stream.write_all(b"\r\n").await?;
            stream.write_all(s.as_bytes()).await?;
            stream.write_all(b"\r\n").await?;
        }
        Ok(())
    }

    pub async fn write_all(&mut self, b: &[u8]) -> Result<(), Error> {
        let mut stream = self.stream.lock().await;
        stream.write_all(b).await?;
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
            stream: Arc::clone(&self.stream),
            buffer: BytesMut::with_capacity(1024),
        }
    }
}
