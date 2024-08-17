use anyhow::Result;
use bytes::BytesMut;
use std::fs::File;
use std::io::Read;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::command_handler::COMMAND_HANDLERS;
use crate::model::Value;
use crate::server::Server;
use crate::utilities::extract_command;

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

fn parse_message(buffer: BytesMut) -> Result<(Value, usize)> {
    match buffer[0] as char {
        '+' => parse_simple_string(buffer),
        '*' => parse_array(buffer),
        '$' => parse_bulk_string(buffer),
        _ => Err(anyhow::anyhow!("Unknown value type {:?}", buffer)),
    }
}

fn parse_simple_string(buffer: BytesMut) -> Result<(Value, usize)> {
    if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let string = String::from_utf8(line.to_vec()).unwrap();

        return Ok((Value::SimpleString(string), len + 1));
    }

    return Err(anyhow::anyhow!("Invalid string {:?}", buffer));
}

fn parse_array(buffer: BytesMut) -> Result<(Value, usize)> {
    let (array_length, mut bytes_consumed) =
        if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
            let array_length = parse_int(line)?;

            (array_length, len + 1)
        } else {
            return Err(anyhow::anyhow!("Invalid array format {:?}", buffer));
        };

    let mut items = vec![];
    for _ in 0..array_length {
        let (array_item, len) = parse_message(BytesMut::from(&buffer[bytes_consumed..]))?;

        items.push(array_item);
        bytes_consumed += len
    }

    return Ok((Value::Array(items), bytes_consumed));
}

fn parse_bulk_string(buffer: BytesMut) -> Result<(Value, usize)> {
    let (bulk_str_len, bytes_consumed) = if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let bulk_str_len = parse_int(line)?;

        (bulk_str_len, len + 1)
    } else {
        return Err(anyhow::anyhow!("Invalid array format {:?}", buffer));
    };

    let end_of_bulk_str = bytes_consumed + bulk_str_len as usize;
    let total_parsed = end_of_bulk_str + 2;

    Ok((
        Value::BulkString(String::from_utf8(
            buffer[bytes_consumed..end_of_bulk_str].to_vec(),
        )?),
        total_parsed,
    ))
}

fn read_until_crlf(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buffer.len() {
        if buffer[i - 1] == b'\r' && buffer[i] == b'\n' {
            return Some((&buffer[0..(i - 1)], i + 1));
        }
    }

    return None;
}

fn parse_int(buffer: &[u8]) -> Result<i64> {
    Ok(String::from_utf8(buffer.to_vec())?.parse::<i64>()?)
}
