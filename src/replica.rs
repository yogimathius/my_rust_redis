use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::log;
use crate::server::Server;
use crate::utilities::ServerState;

pub struct ReplicaClient {
    pub port: u16,
    pub stream: TcpStream,
    pub handshakes: u8,
    pub sync: bool,
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
            sync: false,
        })
    }

    pub async fn send_ping(&mut self, server: &Server) -> Result<()> {
        let msg = server.send_ping().unwrap();
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
        let msg = server.send_psync().unwrap();
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
        server: &mut Server,
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
            _ if response.starts_with("+FULLRESYNC") => {
                log!("ready for rdbsync");
                server.server_state = ServerState::ReceivingRdbDump;
                return Ok(());
            }
            _ => match server.server_state {
                ServerState::ReceivingRdbDump => {
                    log!("Receiving RDB dump");
                    server.server_state = ServerState::AwaitingGetAck;
                    return Ok(());
                }
                ServerState::AwaitingGetAck => {
                    log!("Received get ack");
                    self.sync = true;
                    server.server_state = ServerState::StreamingCommands;
                    return Ok(());
                }
                _ => {
                    log!("Unknown response: {}", response);
                }
            },
        }
        Ok(())
    }
}
