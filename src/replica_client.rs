use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::server::Server;
use crate::Args;

pub struct ReplicaClient {
    pub stream: TcpStream,
}

impl ReplicaClient {
    pub async fn new(vec: Vec<String>) -> Result<Self> {
        let mut iter = vec.into_iter();
        let addr = iter.next().unwrap();
        let port = iter.next().unwrap();
        let stream = TcpStream::connect(format!("{addr}:{port}")).await.unwrap();

        Ok(Self { stream })
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
}
