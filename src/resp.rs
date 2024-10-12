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
            if let Some(value) = self.read_value().await? {
                log!("value: {:?}", value);
                let response = self.process_command(value, &mut server)?;
                if let Some(response) = response {
                    log!("response: {:?}", response);
                    self.write_value(response).await?;
                }
            }

            if server.sync {
                self.handle_sync(&mut server).await?;
            }
        }
    }

    fn process_command(&self, value: Value, server: &mut Server) -> Result<Option<Value>> {
        match value {
            Value::Error(err) => Ok(Some(Value::Error(err))),
            _ => self.execute_command(value, server),
        }
    }

    fn execute_command(&self, value: Value, server: &mut Server) -> Result<Option<Value>> {
        match extract_command(value) {
            Ok((command, key, args)) => {
                if command == "FULLRESYNC" {
                    server.sync = true;
                    Ok(Some(Value::SimpleString("OK".to_string())))
                } else if let Some(command_function) = COMMAND_HANDLERS.get(command.as_str()) {
                    log!("command: {}", command);
                    Ok(command_function(server, key, args))
                } else {
                    Ok(Some(Value::Error("Unknown command".to_string())))
                }
            }
            Err(e) => Ok(Some(Value::Error(e.to_string()))),
        }
    }

    async fn handle_sync(&mut self, server: &mut Server) -> Result<()> {
        log!("server synced");

        let mut rdb_buf: Vec<u8> = vec![];
        let _ = File::open("rdb").await?.read_to_end(&mut rdb_buf).await?;
        log!("Read {} bytes from dump.rdb", rdb_buf.len());
        let contents = hex::decode(&rdb_buf)?;
        let header = format!("${}\r\n", contents.len());
        self.stream.write_all(header.as_bytes()).await?;
        self.stream.write_all(&contents).await?;

        let replconf = server
            .generate_replconf("REPLCONF", vec![("GETACK", "1".to_string())])
            .unwrap();
        self.stream
            .write_all(replconf.serialize().as_bytes())
            .await?;
        server.sync = false;

        Ok(())
    }

    pub async fn read_value(&mut self) -> Result<Option<Value>> {
        let bytes_read = self.stream.read_buf(&mut self.buffer).await?;

        if bytes_read == 0 {
            return Ok(None);
        }

        match parse_message(self.buffer.split()) {
            Ok((v, _)) => Ok(Some(v)),
            Err(e) => Ok(Some(Value::Error(e.to_string()))),
        }
    }

    pub async fn write_value(&mut self, value: Value) -> Result<()> {
        self.stream.write(value.serialize().as_bytes()).await?;

        Ok(())
    }
}
