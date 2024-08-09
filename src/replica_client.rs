use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::server::Server;

pub struct ReplicaClient {
    stream: TcpStream,
}

impl ReplicaClient {
    pub async fn new(address: &str) -> Result<Self> {
        let stream = TcpStream::connect(address).await?;
        Ok(Self { stream })
    }

    pub async fn send_ping(&mut self, server: &Server) -> Result<()> {
        let msg = server.ping().unwrap();
        self.stream.write_all(msg.serialize().as_bytes()).await?;

        // Wait for PONG response
        let mut buffer = [0; 512];
        let n = self.stream.read(&mut buffer).await?;
        let response = String::from_utf8_lossy(&buffer[..n]);
        if response.trim() != "PONG" {
            return Err(anyhow::anyhow!("Expected PONG, got {}", response));
        }

        Ok(())
    }

    pub async fn send_replconf(&mut self, server: &Server) -> Result<()> {
        let replconf = server.replconf().unwrap();
        self.stream
            .write_all(replconf.serialize().as_bytes())
            .await?;
        Ok(())
    }
}
