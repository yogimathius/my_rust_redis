use std::sync::Arc;
use std::vec;

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::log;
use crate::models::value::Value;
use crate::server::{Server, ServerState};

pub struct ReplicaClient {
    pub port: u16,
    pub stream: Arc<Mutex<TcpStream>>,
    pub handshakes: u8,
    pub sync: bool,
}

impl ReplicaClient {
    pub async fn new(stream: Arc<Mutex<TcpStream>>, port: u16) -> Result<Self> {
        Ok(Self {
            port: port,
            stream,
            handshakes: 0,
            sync: false,
        })
    }

    pub async fn send_ping(&mut self) -> Result<()> {
        let msg = Value::Array(vec![Value::BulkString(String::from("PING").into())]);
        self.stream
            .lock()
            .await
            .write_all(msg.serialize().as_bytes())
            .await?;
        Ok(())
    }

    pub async fn send_replconf(&mut self, server: &Server) -> Result<()> {
        let message = vec![Value::BulkString("REPLCONF".into())];
        log!("server state: {:?}", server.state);
        let secondary = match self.handshakes {
            1 => {
                vec![
                    Value::BulkString("listening-port".into()),
                    Value::BulkString(self.port.to_string()),
                ]
            }
            2 => {
                vec![
                    Value::BulkString("capa".into()),
                    Value::BulkString("psync2".into()),
                ]
            }
            _ => vec![],
        };

        let replconf = Value::Array([message, secondary].concat());
        log!("Sending REPLCONF: {:?}", replconf);
        self.stream
            .lock()
            .await
            .write_all(replconf.serialize().as_bytes())
            .await?;
        Ok(())
    }

    pub async fn send_psync(&mut self, _: &Server) -> Result<()> {
        let msg = Value::Array(vec![
            Value::BulkString("PSYNC".into()),
            Value::BulkString("?".into()),
            Value::BulkString("-1".into()),
        ]);
        self.stream
            .lock()
            .await
            .write_all(msg.serialize().as_bytes())
            .await?;
        Ok(())
    }

    pub async fn read_response(&mut self) -> Result<String, std::io::Error> {
        let mut buffer = [0; 512];
        let n = self.stream.lock().await.read(&mut buffer).await?;
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
                server.state = ServerState::SendingCapabilities;
            }
            "+OK" => {
                if self.handshakes == 3 {
                    self.send_psync(server).await?;
                } else {
                    self.send_replconf(server).await?;
                    if self.handshakes == 2 {
                        log!("3 handshakes completed");
                        server.state = ServerState::AwaitingFullResync;
                    }
                }
            }
            _ if response.starts_with("+FULLRESYNC") => {
                log!("ready for rdbsync");
                server.state = ServerState::ReceivingRdbDump;
                return Ok(());
            }
            _ => match server.state {
                ServerState::ReceivingRdbDump => {
                    log!("Receiving RDB dump");
                    // server.state = ServerState::AwaitingGetAck;
                    return Ok(());
                }
                // ServerState::AwaitingGetAck => {
                //     log!("Received get ack");
                //     self.sync = true;
                //     server.state = ServerState::StreamingCommands;
                //     return Ok(());
                // }
                _ => {
                    log!("Unknown response: {}", response);
                }
            },
        }
        Ok(())
    }
}
