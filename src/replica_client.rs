use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::server::Server;

pub struct ReplicaClient {
    pub port: u16,
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
            port: port.parse::<u16>().unwrap(),
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
        let command = "REPLCONF";
        let params = match self.handshakes {
            1 => vec![("listening-port", server.port.to_string())],
            2 => vec![("capa", "psync2".to_string())],
            _ => vec![],
        };
        let replconf = server.generate_replconf(command, params).unwrap();
        self.stream
            .write_all(replconf.serialize().as_bytes())
            .await?;
        Ok(())
    }

    pub async fn send_psync(&mut self, server: &Server) -> Result<()> {
        let msg = server.psync().unwrap();
        self.stream.write_all(msg.serialize().as_bytes()).await?;
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

    pub async fn handle_response(
        &mut self,
        response: &str,
        server: &Server,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match response.trim() {
            "+PONG" => {
                self.send_replconf(server).await?;
            }
            "+OK" => {
                if self.handshakes == 3 {
                    self.send_psync(server).await?;
                } else {
                    self.send_replconf(server).await?;
                }
            }
            _ => {
                println!("Failed to establish replication: {}", response);
            }
        }
        Ok(())
    }
}
