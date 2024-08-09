use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::server::Server;
use crate::Args;

pub struct ReplicaClient {
    pub stream: TcpStream,
    pub handshakes: u8,
}

impl ReplicaClient {
    pub async fn new(vec: Vec<String>) -> Result<Self> {
        let mut iter = vec.into_iter();
        let addr = iter.next().unwrap();
        let port = iter.next().unwrap();
        let stream = TcpStream::connect(format!("{addr}:{port}")).await.unwrap();

        Ok(Self {
            stream,
            handshakes: 0,
        })
    }

    pub async fn send_ping(&mut self, server: &Server) -> Result<()> {
        let msg = server.ping().unwrap();
        self.stream.write_all(msg.serialize().as_bytes()).await?;
        Ok(())
    }

    pub async fn send_replconf(&mut self, server: &Server) -> Result<()> {
        let replconf = server.replconf().unwrap();
        self.stream
            .write_all(replconf.serialize().as_bytes())
            .await?;
        Ok(())
    }

    pub async fn read_response(&mut self) -> Result<String, std::io::Error> {
        let mut buffer = [0; 512];
        let n = self.stream.read(&mut buffer).await?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Connection closed by the server",
            ));
        }
        Ok(String::from_utf8_lossy(&buffer[..n]).to_string())
    }
}
