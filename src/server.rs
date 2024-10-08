use bytes::{Buf, Bytes, BytesMut};
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::command::{Command, RedisCommand};
use crate::log;
use crate::models::connection_state::ConnectionState;
use crate::models::message::Message;
use crate::models::redis_item::RedisItem;
use crate::models::redis_type::RedisType;
use crate::models::role::Role;
use crate::models::value::Value;
use crate::replica::ReplicaClient;
use crate::utilities::parse_message;

#[derive(Debug, Clone, PartialEq)]
pub enum ServerState {
    Initialising,
    SendingHandshake,
    AwaitingReplConfOk,
    SendingCapabilities,
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
    pub data: HashMap<String, RedisItem>,
    pub sync: bool,
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
            sync: false,
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

        let stream_clone = Arc::clone(&connection.stream);

        let mut replica = ReplicaClient::new(stream_clone, self.port).await.unwrap();
        replica.send_ping().await.unwrap();
        self.state = ServerState::SendingHandshake;
        while replica.sync == false {
            match replica.read_response().await {
                Ok(response) => {
                    log!("response in match: {}", response);
                    replica.handshakes += 1;
                    replica.handle_response(&response, self).await.unwrap();
                }
                Err(e) => {
                    log!("Failed to read from stream: {}", e);
                }
            }
        }
    }

    pub async fn read_response(&mut self) -> Result<String, std::io::Error> {
        let mut buffer = [0; 512];
        let n = self
            .master_connection
            .as_mut()
            .unwrap()
            .stream
            .lock()
            .await
            .read(&mut buffer)
            .await?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Connection closed by the server",
            ));
        }
        Ok(String::from_utf8_lossy(&buffer[..n]).to_string())
    }

    pub async fn rdb_dump(&self) -> Result<(), Box<dyn std::error::Error>> {
        let data = self.serialize_data()?; // Implement this method to serialize your data
        let mut file = File::create("dump.rdb")?;
        file.write_all(&data)?;
        Ok(())
    }

    fn serialize_data(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = &self.data;
        let mut buffer = Vec::new();

        // Write header
        buffer.extend_from_slice(b"REDIS0006"); // Example header with version

        // Write each key-value pair
        for (key, value) in data.iter() {
            buffer.push(0x01); // Type byte for string
            buffer.extend_from_slice(&(key.len() as u32).to_be_bytes());
            buffer.extend_from_slice(key.as_bytes());
            let serialized_value = value.serialize().expect("Failed to serialize value");
            buffer.extend_from_slice(&(serialized_value.len() as u32).to_be_bytes());
            buffer.extend_from_slice(&serialized_value);
        }

        // Write footer
        buffer.push(0xFF); // End of file marker

        Ok(buffer)
    }
}
