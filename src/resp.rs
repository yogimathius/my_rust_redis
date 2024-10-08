use std::sync::Arc;
use std::vec;
use tokio::sync::{broadcast, Mutex};

use anyhow::Result;
use bytes::BytesMut;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::commands::COMMAND_HANDLERS;
use crate::log;
use crate::models::value::Value;
use crate::server::Server;
use crate::utilities::{extract_command, parse_message, unpack_bulk_str};

pub struct RespHandler {
    stream: TcpStream,
    buffer: BytesMut,
    // server: Arc<Mutex<Server>>,
}

impl RespHandler {
    pub fn new(stream: TcpStream, _: Arc<Mutex<Server>>) -> Self {
        RespHandler {
            stream,
            buffer: BytesMut::with_capacity(512),
            // server,
        }
    }

    pub async fn handle_client(&mut self, server: Arc<Mutex<Server>>) -> Result<()> {
        loop {
            let value: Option<Value> = self.read_value().await?;

            let response: Option<Value> = if let Some(value) = value {
                log!("value: {:?}", value);
                self.execute_command(value, server.clone()).await
            } else {
                None
            };
            if let Some(response) = response {
                self.write_value(response).await.unwrap();
            }
            let mut server = server.lock().await;
            if server.sync {
                log!("server synced in resp");
                self.write_rdb().await.unwrap();
                server.sync = false;
                // let mut receiver = sender.subscribe();
                // while let Ok(f) = receiver.recv().await {
                //     self.stream.write_all(&f.serialize().as_bytes()).await?;
                // }
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

    pub async fn execute_command(
        &mut self,
        value: Value,
        server: Arc<Mutex<Server>>,
    ) -> Option<Value> {
        let (command, args) = extract_command(&value.clone()).unwrap();
        let key = unpack_bulk_str(args.get(0).unwrap()).unwrap();
        if command == "SET" {
            println!("SET command");
        }
        if command == "PSYNC" {}
        log!("command: {}", command);
        if let Some(command_function) = COMMAND_HANDLERS.get(command.as_str()) {
            command_function
                .handle(server.clone().lock().await.data.clone(), key, args)
                .await
        } else {
            Value::SimpleString("Unknown command".to_string());
            std::process::exit(1);
        }
    }

    pub async fn write_rdb(&mut self) -> Result<()> {
        let mut rdb_buf: Vec<u8> = vec![];
        log!("Opening dump.rdb");
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

        Ok(())
    }
}
