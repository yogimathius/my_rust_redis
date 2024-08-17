use anyhow::Result;
use bytes::BytesMut;
use std::fs::File;
use std::io::Read;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::command_handler::COMMAND_HANDLERS;
use crate::model::Value;
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
                let (command, args) = extract_command(value).unwrap();

                if let Some(command_function) = COMMAND_HANDLERS.get(command.as_str()) {
                    command_function(&mut server, args)
                } else {
                    Value::SimpleString("Unknown command".to_string());
                    std::process::exit(1);
                }
            } else {
                Some(Value::SimpleString("Unknown command".to_string()))
            };
            self.write_value(response.unwrap()).await.unwrap();
            if server.sync {
                let mut file = File::open("dump.rdb").unwrap();
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).unwrap();

                self.write_value(Value::BulkString(String::from_utf8(buffer).unwrap()))
                    .await
                    .unwrap();

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
