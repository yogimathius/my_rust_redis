// use bytes::Buf;
use bytes::BytesMut;
use regex::Regex;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::command::{Command, RedisCommand};
use crate::log;
use crate::models::connection_state::ConnectionState;
use crate::models::data_value::DataValue;
use crate::models::message::Message;
use crate::models::role::Role;
// use crate::models::value::Value;
// use crate::utilities::{extract_command, parse_message, unpack_bulk_str};

pub struct Server {
    port: u16,
    pub role: Role,
    master_addr: Option<String>,
    master_replid: String,
    master_repl_offset: u32,
    pub listener: Option<Arc<TcpListener>>,
    pub connections: Vec<ConnectionState>,
    master_connection: Option<ConnectionState>,
    messages: VecDeque<Message>,
    replications: HashMap<String, Arc<Mutex<TcpStream>>>,
    data: HashMap<String, DataValue>,
}

impl Server {
    pub fn new(port: u16, replicaof: Option<String>) -> Self {
        let role = match replicaof {
            Some(_) => Role::Slave,
            None => Role::Master,
        };
        let master_addr = if replicaof.is_some() {
            Some(replicaof.unwrap().replace(" ", ":"))
        } else {
            None
        };
        let master_replid = if role == Role::Master {
            "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string()
        } else {
            "?".to_string()
        };
        Server {
            port,
            role,
            master_addr,
            master_connection: None,
            master_replid,
            master_repl_offset: 0,
            listener: None,
            connections: Vec::new(),
            messages: VecDeque::new(),
            replications: HashMap::new(),
            data: HashMap::new(),
        }
    }

    pub async fn run(&mut self) {
        if self.role == Role::Slave {
            self.connect_to_master().await;
        }

        let listener = match TcpListener::bind(format!("127.0.0.1:{}", self.port)).await {
            Ok(listener) => listener,
            Err(e) => panic!("Error binding to port: {}", e),
        };

        self.listener = Some(Arc::new(listener)); // Assign the listener
        log!("Listening on 127.0.0.1:{}...", self.port);
    }

    pub async fn accept_connections(&mut self) {
        log!("Accepting connections...");
        let listener = self.listener.as_ref().unwrap();
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    log!("Accepted connection");
                    log!("Stream: {:?}", stream);
                    let connection_state = ConnectionState::new(stream);
                    self.connections.push(connection_state);
                }
                Err(e) => {
                    panic!("Error accepting connection: {}", e);
                }
            }
            tokio::task::yield_now().await;
        }
    }

    pub async fn read_messages(&mut self) {
        let mut closed_indices = Vec::new();
        for (index, connection) in self.connections.iter_mut().enumerate() {
            let (should_close, new_commands) = Server::read_messages_from(&connection.stream).await;
            if should_close {
                closed_indices.push(index);
            }
            log!("Received {} commands from client", new_commands.len());
            self.messages
                .extend(new_commands.into_iter().map(|command| Message {
                    connection: Arc::clone(&connection.stream),
                    command,
                }));
        }
        for &index in closed_indices.iter().rev() {
            self.connections.remove(index);
        }
        if let Some(master_connection) = self.master_connection.as_mut() {
            let (_should_close, new_commands) =
                Server::read_messages_from(&master_connection.stream).await;
            log!("Received {} commands from master", new_commands.len());
            if new_commands.len() > 0 {
                log!("Received {} commands from master", new_commands.len());
                self.messages
                    .extend(new_commands.into_iter().map(|command| Message {
                        connection: Arc::clone(&master_connection.stream),
                        command,
                    }));
            }
        }
    }

    async fn read_messages_from(stream: &Arc<Mutex<TcpStream>>) -> (bool, Vec<Command>) {
        log!("Reading messages from client...");
        let mut buf = BytesMut::with_capacity(1024);
        let mut new_commands: Vec<Command> = Vec::new();
        let mut should_close = false;

        let mut stream = stream.lock().await;

        match stream.read_buf(&mut buf).await {
            Ok(0) => {
                log!("Read 0 bytes, closing connection");
                should_close = true;
            }
            Ok(bytes_read) => {
                log!("Read {} bytes", bytes_read);
                let command = Command::parse(&mut buf).await.unwrap();
                new_commands.extend(vec![command]);
                log!("Added commands to new_commands: {:?}", new_commands);
            }
            Err(e) => {
                log!("Error reading from stream: {:?}", e);
                should_close = true;
            }
        }

        log!("Finished reading messages from client: {:?}", new_commands);
        (should_close, new_commands)
    }

    pub async fn process_messages(&mut self) {
        while let Some(message) = self.messages.pop_front() {
            log!("Processing message: {:?}", message);
            let mut stream = message.connection.lock().await;

            match message.command.command {
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
                    let mut datavalue = DataValue {
                        value,
                        expiry: None,
                    };
                    if let Some(expiry_time) = expiry_time {
                        if let Some(flag) = expiry_flag {
                            if flag.to_uppercase() == "PX" {
                                let expiry_timestamp = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis()
                                    + expiry_time as u128;
                                datavalue.expiry = Some(expiry_timestamp);
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
                    Some(value) => match value.expiry {
                        Some(expiry) => {
                            let current_timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis();
                            if expiry < current_timestamp {
                                self.data.remove(&key);
                                let response = "$-1\r\n";
                                stream.write_all(response.as_bytes()).await.unwrap();
                            } else {
                                let response = format!("+{}\r\n", value.value);
                                stream.write_all(response.as_bytes()).await.unwrap();
                            }
                        }
                        None => {
                            let response = format!("+{}\r\n", value.value);
                            stream.write_all(response.as_bytes()).await.unwrap();
                        }
                    },
                    None => {
                        let response = "$-1\r\n";
                        stream.write_all(response.as_bytes()).await.unwrap();
                    }
                },
                RedisCommand::Command => {
                    let response = "+OK\r\n";
                    stream.write_all(response.as_bytes()).await.unwrap();
                }
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
                }
                RedisCommand::ReplConfGetAck => {
                    // Send REPLCONF ACK 0
                    let response = "*3\r\n$8\r\nREPLCONF\r\n$3\r\nACK\r\n$1\r\n0\r\n";
                    let mut stream = message.connection.lock().await;
                    stream.write_all(response.as_bytes()).await.unwrap();
                }
                RedisCommand::FullResync(_id, _offset) => {
                    log!("id: {}, offset: {}", _id, _offset);
                }
                RedisCommand::Rdb(_bytes) => {
                    log!("Received RDB");
                }
            }
        }
    }

    async fn connect_to_master(&mut self) {
        let addr = self.master_addr.as_ref().unwrap();
        match TcpStream::connect(addr).await {
            Ok(stream) => {
                let connection = ConnectionState::new(stream);
                self.master_connection = Some(connection);
            }
            Err(e) => {
                panic!("Error connecting to master: {}", e);
            }
        }
        let connection = self.master_connection.as_ref().unwrap();
        log!("Sending PING to master...");
        let message = format!("*1\r\n$4\r\nPING\r\n");
        match connection
            .stream
            .lock()
            .await
            .write_all(message.as_bytes())
            .await
        {
            Ok(_) => {}
            Err(e) => {
                panic!("Error PINGing master: {}", e);
            }
        }
        self.expect_read(&connection.stream, "+PONG").await;
        log!("Sending REPLCONF listening-port to master...");
        let message = format!(
            "*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n${}\r\n{}\r\n",
            self.port.to_string().len(),
            self.port
        );
        connection
            .stream
            .lock()
            .await
            .write_all(message.as_bytes())
            .await
            .unwrap();
        log!("Checking OK...");
        self.expect_read(&connection.stream, "+OK").await;
        let message = format!("*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n");
        connection
            .stream
            .lock()
            .await
            .write_all(message.as_bytes())
            .await
            .unwrap();
        self.expect_read(&connection.stream, "+OK").await;
        log!("Sending PSYNC to master...");
        let message = "*3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n";
        connection
            .stream
            .lock()
            .await
            .write_all(message.as_bytes())
            .await
            .unwrap();
        let mut buf = BytesMut::with_capacity(1024);
        log!("Reading from master...");
        loop {
            match connection.stream.lock().await.read_buf(&mut buf).await {
                Ok(0) => {
                    log!("Read 0 bytes, closing connection");
                    // should_close = true;
                }
                Ok(bytes_read) => {
                    log!("Read {} bytes", bytes_read);
                    log!("Buffer: {:?}", buf);
                    // parse fullresync string and rdb
                    let fullresync = Regex::new(r"\+FULLRESYNC ([a-z0-9]+) (\d+)\r\n").unwrap();
                    let captures = fullresync.captures(std::str::from_utf8(&buf).unwrap());
                    log!("Captures: {:?}", captures);
                    if let Some(captures) = captures {
                        let id = captures.get(1).unwrap().as_str();
                        let offset = captures.get(2).unwrap().as_str();
                        log!("id: {}, offset: {}", id, offset);

                        // expect rdb
                        log!("captures: {:?}", captures);
                        break;
                    }
                }
                Err(e) => {
                    log!("Error reading from stream: {:?}", e);
                    // should_close = true;
                }
            }
        }
    }

    async fn expect_read(&self, stream: &Arc<Mutex<TcpStream>>, expected: &str) {
        let mut buf = [0; 1024];
        let mut stream = stream.lock().await;
        match stream.read(&mut buf).await {
            Ok(bytes_read) => {
                let response = std::str::from_utf8(&buf[..bytes_read]).unwrap();
                let trimmed = response.trim();
                if trimmed != expected {
                    panic!(
                        "Unexpected response from master: {} (expected {})",
                        trimmed, expected
                    );
                }
                log!("Received expected response from master: {}", expected);
            }
            Err(e) => {
                panic!("Error reading from master: {}", e);
            }
        }
    }
}
