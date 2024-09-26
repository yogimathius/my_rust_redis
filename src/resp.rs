use anyhow::Result;
use bytes::BytesMut;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::commands::COMMAND_HANDLERS;
use crate::log;
use crate::models::value::Value;
use crate::server::Server;
use crate::utilities::{extract_command, parse_message};

pub struct RespHandler {
    stream: TcpStream,
    buffer: BytesMut,
}

impl RespHandler {
    pub fn new(stream: TcpStream) -> Self {
        RespHandler {
            stream,
            buffer: BytesMut::with_capacity(512),
        }
    }

    pub async fn handle_client(&mut self, mut server: Server) -> Result<()> {
        loop {
            let value: Option<Value> = self.read_value().await?;

            let response: Option<Value> = if let Some(value) = value {
                let (command, key, args) = extract_command(value).unwrap();
                if command == "FULLRESYNC" {
                    server.sync = true;
                    Some(Value::SimpleString("OK".to_string()))
                } else if let Some(command_function) = COMMAND_HANDLERS.get(command.as_str()) {
                    command_function(&mut server, key, args)
                } else {
                    Value::SimpleString("Unknown command".to_string());
                    std::process::exit(1);
                }
            } else {
                None
            };
            if let Some(response) = response {
                self.write_value(response).await.unwrap();
            }
            if server.sync {
                log!("server synced");

                let mut rdb_buf: Vec<u8> = vec![];
                let _ = File::open("rdb")
                    .await
                    .unwrap()
                    .read_to_end(&mut rdb_buf)
                    .await;
                log!("Read {} bytes from dump.rdb", rdb_buf.len());
                let contents = hex::decode(&rdb_buf).unwrap();
                let header = format!("${}\r\n", contents.len());
                self.stream.write_all(header.as_bytes()).await?;
                self.stream.write_all(&contents).await?;

                server.sync = false;
            }
        }
    }

    pub async fn read_value(&mut self) -> Result<Option<Value>> {
        let bytes_read = self.stream.read_buf(&mut self.buffer).await?;

        if bytes_read == 0 {
            return Ok(None);
        }

        let (v, _) = parse_message(self.buffer.split())?;

        Ok(Some(v))
    }

    pub async fn write_value(&mut self, value: Value) -> Result<()> {
        self.stream.write(value.serialize().as_bytes()).await?;

        Ok(())
    }
}
