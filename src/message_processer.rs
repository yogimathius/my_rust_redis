use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::command::RedisCommand;
use crate::log;
use crate::models::message::Message;
use crate::models::redis_item::RedisItem;
use crate::models::redis_type::RedisType;
use crate::models::role::Role;
use crate::models::value::Value;
use crate::server::ServerState;

pub struct MessageProcessor {
    pub messages: VecDeque<Message>,
    pub data: HashMap<String, RedisItem>,
    pub role: Role,
    pub replications: HashMap<String, Arc<Mutex<TcpStream>>>,
    pub master_replid: String,
    pub master_repl_offset: u32,
    pub state: ServerState,
}

impl MessageProcessor {
    pub fn new(
        role: Role,
        replications: HashMap<String, Arc<Mutex<TcpStream>>>,
        master_replid: String,
        master_repl_offset: u32,
        state: ServerState,
    ) -> Self {
        MessageProcessor {
            messages: VecDeque::new(),
            data: HashMap::new(),
            role,
            replications,
            master_replid,
            master_repl_offset,
            state,
        }
    }

    pub async fn process_messages(&mut self) {
        while let Some(message) = self.messages.pop_front() {
            log!("Processing message: {:?}", message);
            self.process_command(message).await;
        }
    }

    async fn process_command(&mut self, message: Message) {
        log!("Processing command: {:?}", message.command);
        let mut stream = message.connection.lock().await;

        match message.command.command {
            RedisCommand::FullResync(replid, offset) => {
                self.master_replid = replid;
                self.master_repl_offset = offset as u32;
            }
            RedisCommand::Rdb(data) => {
                self.state = ServerState::ReceivingRdbDump;
                log!("Received RDB data: {:?}", data);
                self.state = ServerState::StreamingCommands;
                // self.data = data;
            }
            RedisCommand::Command => {
                self.state = ServerState::StreamingCommands;
            }
            RedisCommand::Ping => {
                log!("Received PING command");
                let response = "+PONG\r\n";
                stream.write_all(response.as_bytes()).await.unwrap();
            }
            RedisCommand::Echo(content) => {
                let response = format!("+{}\r\n", content);
                stream.write_all(response.as_bytes()).await.unwrap();
            }
            RedisCommand::Set(key, value, expiry_flag, expiry_time) => {
                let mut datavalue = RedisItem {
                    value: Value::BulkString(value),
                    created_at: std::time::Instant::now(),
                    expiration: None,
                    redis_type: RedisType::String,
                };
                if let Some(expiry_time) = expiry_time {
                    if let Some(flag) = expiry_flag {
                        if flag.to_uppercase() == "PX" {
                            let expiry_timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis()
                                + expiry_time as u128;
                            datavalue.expiration = Some(expiry_timestamp.try_into().unwrap());
                        }
                    }
                }
                self.data.insert(key, datavalue);
                if self.role == Role::Master {
                    for (_port, slave) in &self.replications {
                        let mut slave_stream = slave.lock().await;
                        slave_stream
                            .write_all(message.command.raw.as_ref())
                            .await
                            .unwrap();
                    }
                }
                let response = "+OK\r\n";
                stream.write_all(response.as_bytes()).await.unwrap();
            }
            RedisCommand::Get(key) => match self.data.get(&key) {
                Some(value) => match value.expiration {
                    Some(expiration) => {
                        let current_timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                        if expiration < current_timestamp.try_into().unwrap() {
                            self.data.remove(&key);
                            let response = "$-1\r\n";
                            stream.write_all(response.as_bytes()).await.unwrap();
                        } else {
                            let response = format!("+{:?}\r\n", value.value);
                            stream.write_all(response.as_bytes()).await.unwrap();
                        }
                    }
                    None => {
                        let response = format!("+{:?}\r\n", value.value);
                        stream.write_all(response.as_bytes()).await.unwrap();
                    }
                },
                None => {
                    let response = "$-1\r\n";
                    stream.write_all(response.as_bytes()).await.unwrap();
                }
            },
            RedisCommand::Info => {
                let role_field = format!("role:{}", self.role);
                let master_replid_field = format!("master_replid:{}", self.master_replid);
                let master_repl_offset_field =
                    format!("master_repl_offset:{}", self.master_repl_offset);
                let params = format!(
                    "{}\r\n{}\r\n{}",
                    role_field, master_replid_field, master_repl_offset_field
                );
                let length = params.len();
                let response = format!("${}\r\n{}\r\n", length, params);
                stream.write_all(response.as_bytes()).await.unwrap();
            }
            RedisCommand::ReplConf(key, value) => {
                log!("Received REPLCONF command: {} {}", key, value);
                if key == "listening-port" {
                    self.replications
                        .insert(value, Arc::clone(&message.connection));
                }
                let response = "+OK\r\n";
                stream.write_all(response.as_bytes()).await.unwrap();
            }
            RedisCommand::Psync(_id, _offset) => {
                log!("id: {}, offset: {}", _id, _offset);
                let response = "+FULLRESYNC 8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb 0\r\n";
                stream.write_all(response.as_bytes()).await.unwrap();
                let rdb = "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2";
                let mut bytes = Vec::new();
                for i in (0..rdb.len()).step_by(2) {
                    let byte = u8::from_str_radix(&rdb[i..i + 2], 16).unwrap();
                    bytes.push(byte);
                }
                let response = format!("${}\r\n", bytes.len());
                stream.write_all(response.as_bytes()).await.unwrap();
                stream.write_all(&bytes).await.unwrap();
                stream.write_all(b"\r\n").await.unwrap();
            }
            RedisCommand::ReplConfGetAck => {
                // Send REPLCONF ACK 0
                let response = "*3\r\n$8\r\nREPLCONF\r\n$3\r\nACK\r\n$1\r\n0\r\n";
                let mut stream = message.connection.lock().await;
                stream.write_all(response.as_bytes()).await.unwrap();
            }
        }
    }
}
