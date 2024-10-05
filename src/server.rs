use regex::Regex;
use std::collections::{HashMap, VecDeque};
use std::io::{self};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::timeout;

use crate::command::{Command, RedisCommand};
use crate::log;
use crate::models::data_value::DataValue;
use crate::models::message::Message;
use crate::models::role::Role;

pub struct Server {
    port: u16,
    pub role: Role,
    master_addr: Option<String>,
    master_stream: Option<Arc<Mutex<TcpStream>>>,
    master_replid: String,
    master_repl_offset: u32,
    listener: Option<TcpListener>,
    connections: Vec<Arc<Mutex<TcpStream>>>,
    messages: VecDeque<Message>,
    slaves: HashMap<String, Arc<Mutex<TcpStream>>>,
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
            master_stream: None,
            master_replid,
            master_repl_offset: 0,
            listener: None,
            connections: Vec::new(),
            messages: VecDeque::new(),
            slaves: HashMap::new(),
            data: HashMap::new(),
        }
    }

    pub async fn run(&mut self) {
        if self.role == Role::Slave {
            self.connect_to_master().await;
        }

        match TcpListener::bind(format!("127.0.0.1:{}", self.port)).await {
            Ok(listener) => {
                self.listener = Some(listener);
            }
            Err(e) => {
                panic!("Error binding to port: {}", e);
            }
        }
        log!("Listening on 127.0.0.1:{}...", self.port);
    }

    pub async fn accept_connections(&mut self) {
        let listener = self.listener.as_ref().unwrap();
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    self.connections.push(Arc::new(Mutex::new(stream)));
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    break;
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
        for (index, stream) in self.connections.iter_mut().enumerate() {
            let (should_close, new_commands) = Server::read_messages_from(stream).await;
            if should_close {
                closed_indices.push(index);
            }
            log!("Received {} commands from client", new_commands.len());
            self.messages
                .extend(new_commands.into_iter().map(|command| Message {
                    connection: Arc::clone(stream),
                    command,
                }));
        }
        for &index in closed_indices.iter().rev() {
            self.connections.remove(index);
        }
        if let Some(master_stream) = self.master_stream.as_mut() {
            let (_should_close, new_commands) = Server::read_messages_from(master_stream).await;
            log!("Received {} commands from master", new_commands.len());
            if new_commands.len() > 0 {
                log!("Received {} commands from master", new_commands.len());
                self.messages
                    .extend(new_commands.into_iter().map(|command| Message {
                        connection: Arc::clone(master_stream),
                        command,
                    }));
            }
        }
    }

    async fn read_messages_from(stream: &Arc<Mutex<TcpStream>>) -> (bool, Vec<Command>) {
        let mut buf = [0; 1024];
        let mut new_commands: Vec<Command> = Vec::new();
        let mut should_close = false;

        let mut stream = stream.lock().await;

        let result = timeout(Duration::from_secs(10), stream.read(&mut buf)).await;

        match result {
            Ok(Ok(0)) => {
                log!("Read 0 bytes, closing connection");
                should_close = true;
            }
            Ok(Ok(bytes_read)) => {
                log!("Read {} bytes", bytes_read);
                let input = String::from_utf8_lossy(&buf[..bytes_read]);
                log!("Received input: {}", input);
                if input.starts_with("+") {
                    log!("Input starts with '+', continuing...");
                } else {
                    let commands: Vec<Command> = input
                        .split("*")
                        .filter(|s| !s.is_empty())
                        .map(|s| Command::parse(format!("*{}", s).as_str()))
                        .collect();
                    log!("Parsed commands: {:?}", commands);
                    new_commands.extend(commands);
                    log!("Added commands to new_commands: {:?}", new_commands);
                }
            }
            Ok(Err(e)) if e.kind() == io::ErrorKind::WouldBlock => {
                log!("Would block, breaking loop");
            }
            Ok(Err(e)) => {
                log!("Error reading from stream: {:?}", e);
                should_close = true;
            }
            Err(_) => {
                log!("Timeout while reading from stream");
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
                        for (_port, slave) in &self.slaves {
                            let mut slave_stream = slave.lock().await;
                            slave_stream
                                .write_all(message.command.raw.as_bytes())
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
                    if key == "listening-port" {
                        self.slaves.insert(value, Arc::clone(&message.connection));
                    }
                    let response = "+OK\r\n";
                    stream.write_all(response.as_bytes()).await.unwrap();
                }
                RedisCommand::Psync(_id, _offset) => {
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

                    let ack = "+REPLCONF ACK $1 0\r\n";
                    log!("Sending ACK to master: {}", ack);
                    stream.write_all(ack.as_bytes()).await.unwrap();
                }
            }
        }
    }

    async fn connect_to_master(&mut self) {
        let addr = self.master_addr.as_ref().unwrap();
        match TcpStream::connect(addr).await {
            Ok(stream) => {
                self.master_stream = Some(Arc::new(Mutex::new(stream)));
            }
            Err(e) => {
                panic!("Error connecting to master: {}", e);
            }
        }
        let stream = self.master_stream.as_ref().unwrap();
        log!("Sending PING to master...");
        let message = format!("*1\r\n$4\r\nPING\r\n");
        match stream.lock().await.write_all(message.as_bytes()).await {
            Ok(_) => {}
            Err(e) => {
                panic!("Error PINGing master: {}", e);
            }
        }
        self.expect_read(&stream, "+PONG").await;
        log!("Sending REPLCONF listening-port to master...");
        let message = format!(
            "*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n${}\r\n{}\r\n",
            self.port.to_string().len(),
            self.port
        );
        stream
            .lock()
            .await
            .write_all(message.as_bytes())
            .await
            .unwrap();
        self.expect_read(&stream, "+OK").await;
        let message = format!("*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n");
        stream
            .lock()
            .await
            .write_all(message.as_bytes())
            .await
            .unwrap();
        self.expect_read(&stream, "+OK").await;
        let message = "*3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n";
        stream
            .lock()
            .await
            .write_all(message.as_bytes())
            .await
            .unwrap();
        let mut buf = [0; 1024];
        match stream.lock().await.read(&mut buf).await {
            Ok(bytes_read) => {
                let response = String::from_utf8_lossy(&buf[..bytes_read]);
                log!("Received from master: {}", response);
                if !response.starts_with("+FULLRESYNC") {
                    panic!("Unexpected response from master: {}", response);
                }
                let re = Regex::new(r"^\+FULLRESYNC (\S+) (\d+)\r\n").unwrap();
                if let Some(captures) = re.captures(&response) {
                    self.master_replid = captures[1].to_string();
                    match captures[2].parse() {
                        Ok(offset) => {
                            self.master_repl_offset = offset;
                        }
                        Err(e) => {
                            panic!("Error parsing offset: {}", e);
                        }
                    }
                } else {
                    panic!("Unexpected response format from master: {}", response);
                }
            }
            Err(e) => {
                panic!("Error reading from master: {}", e);
            }
        }
        match stream.lock().await.read(&mut buf).await {
            Ok(_) => {
                // TODO: Parse response and load RDB file into datastore
            }
            Err(e) => {
                panic!("Error reading from master: {}", e);
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
            }
            Err(e) => {
                panic!("Error reading from master: {}", e);
            }
        }
    }
}
