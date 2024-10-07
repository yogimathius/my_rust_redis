// use bytes::Buf;
use bytes::{Buf, Bytes, BytesMut};
use core::str;
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
use crate::models::value::Value;
use crate::utilities::parse_message;
// use crate::models::value::Value;
// use crate::utilities::{extract_command, parse_message, unpack_bulk_str};

#[derive(Debug, Clone, PartialEq)]
pub enum ServerState {
    Initialising,
    AwaitingFullResync,
    ReceivingRdbDump,
    StreamingCommands,
}

pub struct Server {
    port: u16,
    pub state: ServerState,
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
            state: ServerState::Initialising,
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
        self.state = ServerState::StreamingCommands;
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
            let (should_close, new_commands) =
                Server::read_messages_from(self.state.clone(), &connection.stream).await;
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
                Server::read_messages_from(self.state.clone(), &master_connection.stream).await;
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

    async fn read_messages_from(
        server_state: ServerState,
        stream: &Arc<Mutex<TcpStream>>,
    ) -> (bool, Vec<Command>) {
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
                log!("Buffer: {:?}", buf);
                log!("Server state: {:?}", server_state);
                if server_state == ServerState::ReceivingRdbDump {
                    log!("Received RDB data");
                    loop {
                        if buf.is_empty() {
                            break;
                        }
                        match parse_message(buf.clone().split()) {
                            Ok((value, bytes_consumed)) => {
                                match value {
                                    Value::BulkString(data) => {
                                        // Advance the buffer by the bytes consumed
                                        buf.advance(bytes_consumed);
                                        new_commands.push(Command::new(
                                            RedisCommand::Rdb(data.into()),
                                            String::new(),
                                            vec![],
                                            Bytes::new(), // You can store raw bytes if needed
                                        ));
                                        return (true, new_commands);
                                    }
                                    Value::Array(_) => {
                                        buf.advance(bytes_consumed);

                                        let command =
                                            Command::parse(&mut buf, server_state).await.unwrap();

                                        new_commands.push(command);
                                        return (true, new_commands);
                                    }
                                    _ => {
                                        log!("TODO: Handle unexpected value types");
                                        buf.advance(bytes_consumed);
                                        return (true, vec![]);
                                    }
                                }
                            }
                            Err(ref e) if e.to_string() == "Incomplete" => {
                                // Need more data to complete the message
                                log!("Incomplete RDB data, waiting for more data");
                                return (false, vec![]); // Don't close the connection
                            }
                            Err(e) => {
                                // Handle parsing error
                                log!("Error parsing RDB data: {:?}", e);
                                return (true, vec![]); // Close the connection or handle error
                            }
                        }
                    }
                } else {
                    let command = Command::parse(&mut buf, server_state).await.unwrap();
                    new_commands.extend(vec![command]);
                    log!("Added commands to new_commands: {:?}", new_commands);
                }
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
            self.process_command(message).await;
        }
    }

    async fn process_command(&mut self, message: Message) {
        log!("Processing command: {:?}", message.command);
        let mut stream = message.connection.lock().await;

        match message.command.command {
            RedisCommand::FullResync(replid, offset) => {
                self.state = ServerState::AwaitingFullResync;
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
        let mut buf = BytesMut::with_capacity(4096);
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

                    if let Some(pos) = buf.windows(2).position(|window| window == b"\r\n") {
                        let line_end = pos + 2; // Position after "\r\n"
                        let line = &buf[..line_end];

                        // Convert only the line to a UTF-8 string
                        match str::from_utf8(line) {
                            Ok(line_str) => {
                                log!("Line: {}", line_str);
                                // Apply the regex to the line
                                let fullresync =
                                    Regex::new(r"^\+FULLRESYNC ([a-z0-9]+) (\d+)\r\n$").unwrap();
                                if let Some(captures) = fullresync.captures(line_str) {
                                    let id = captures.get(1).unwrap().as_str();
                                    let offset = captures.get(2).unwrap().as_str();
                                    log!("id: {}, offset: {}", id, offset);
                                    log!("line_end: {}", line_end);
                                    // Advance the buffer by the length of the line
                                    buf.advance(line_end);
                                    self.state = ServerState::ReceivingRdbDump;
                                    // } else {
                                    // check for rdb and getack here
                                    let (value, _) = parse_message(buf.clone().split()).unwrap();
                                    log!("value: {:?}", value);
                                    match value {
                                        Value::BulkString(data) => {
                                            log!("Received RDB data: {:?}", data);
                                            buf.advance(line_end);
                                            self.state = ServerState::StreamingCommands;
                                        }
                                        Value::Array(_) => {
                                            log!("Received REPLCONF GETACK");
                                            buf.advance(line_end);
                                            let command =
                                                Command::parse(&mut buf, self.state.clone())
                                                    .await
                                                    .unwrap();
                                            log!("command: {:?}", command);
                                            self.messages.extend(vec![Message {
                                                connection: Arc::clone(&connection.stream),
                                                command,
                                            }]);
                                        }
                                        _ => {
                                            log!("TODO: Handle unexpected value types");
                                            buf.advance(line_end);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                log!("Error converting line to UTF-8: {:?}", e);
                                // Handle UTF-8 conversion error
                                return log!("Invalid UTF-8 in FULLRESYNC line");
                            }
                        }
                    } else {
                        // "\r\n" not found; the line might be incomplete
                        log!("Incomplete FULLRESYNC line; need more data");
                        // Wait for more data
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
