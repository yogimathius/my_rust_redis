use regex::Regex;

use crate::{
    log,
    models::{redis_type::RedisType, value::Value},
};

use super::command::{Command, RedisCommand};
use std::{
    collections::{HashMap, VecDeque},
    fmt,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    time::Instant,
};

#[derive(Debug, PartialEq)]
pub struct RedisItem {
    pub value: Value,
    pub created_at: Instant,
    pub expiration: Option<i64>,
    pub redis_type: RedisType,
}

#[derive(Debug)]
struct DataValue {
    value: String,
    expiry: Option<u128>,
}
#[derive(Debug)]
pub struct Message {
    pub connection: TcpStream,
    pub command: Command,
}
#[derive(Debug, PartialEq)]
pub enum Role {
    Master,
    Slave,
}
impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Role::Master => write!(f, "master"),
            Role::Slave => write!(f, "slave"),
        }
    }
}
pub struct RedisServer {
    port: u16,
    role: Role,
    master_addr: Option<String>,
    master_stream: Option<TcpStream>,
    master_replid: String,
    master_repl_offset: u32,
    listener: Option<TcpListener>,
    connections: Vec<TcpStream>,
    messages: VecDeque<Message>,
    slaves: HashMap<String, TcpStream>,
    data: HashMap<String, DataValue>,
}
impl RedisServer {
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
        RedisServer {
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
    pub fn run(&mut self) {
        // If this replica is a slave, connect to the master
        if self.role == Role::Slave {
            self.connect_to_master();
        }
        // Start listening for connections
        match TcpListener::bind(format!("127.0.0.1:{}", self.port)) {
            Ok(listener) => {
                listener.set_nonblocking(true).unwrap();
                self.listener = Some(listener);
            }
            Err(e) => {
                panic!("Error binding to port: {}", e);
            }
        }
        println!("Listening on 127.0.0.1:{}...", self.port);
    }
    pub fn accept_connections(&mut self) {
        let listener = self.listener.as_ref().unwrap();
        loop {
            match listener.accept() {
                Ok(stream) => {
                    let stream = stream.0;
                    stream.set_nonblocking(true).unwrap();
                    self.connections.push(stream);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    panic!("Error accepting connection: {}", e);
                }
            }
        }
    }
    pub fn read_messages(&mut self) {
        let mut closed_indices = Vec::new();
        for (index, stream) in self.connections.iter_mut().enumerate() {
            let (should_close, new_commands) = RedisServer::read_messages_from(stream);
            if should_close {
                closed_indices.push(index);
            }
            self.messages
                .extend(new_commands.into_iter().map(|command| Message {
                    connection: stream.try_clone().unwrap(),
                    command,
                }));
        }
        // Remove closed connections
        for &index in closed_indices.iter().rev() {
            self.connections.remove(index);
        }
        // Read from the master stream if it exists
        if let Some(master_stream) = self.master_stream.as_mut() {
            let (_should_close, new_commands) = RedisServer::read_messages_from(master_stream);
            if new_commands.len() > 0 {
                println!("Received {} commands from master", new_commands.len());
                self.messages
                    .extend(new_commands.into_iter().map(|command| Message {
                        connection: master_stream.try_clone().unwrap(),
                        command,
                    }));
            }
        }
    }
    fn read_messages_from(stream: &mut TcpStream) -> (bool, Vec<Command>) {
        let mut buf = [0; 1024];
        let mut new_commands: Vec<Command> = Vec::new();
        // Read from the connection until there are no more messages
        let mut should_close = false;
        loop {
            match stream.read(&mut buf) {
                Ok(0) => {
                    should_close = true;
                    break;
                }
                Ok(bytes_read) => {
                    // Convert the buffer to a string
                    let input = String::from_utf8_lossy(&buf[..bytes_read]);
                    // If the input starts with "+", it's an ack from a slave. Ignore it.
                    if input.starts_with("+") {
                        continue;
                    }
                    // Split the input into commands (commands begin with *) and convert to command
                    let commands = input
                        .split("*")
                        .filter(|s| s.len() > 0)
                        .map(|s| Command::parse(format!("*{}", s).as_str()));
                    new_commands.extend(commands);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(_) => {
                    should_close = true;
                    break;
                }
            }
        }
        return (should_close, new_commands);
    }
    pub fn process_messages(&mut self) {
        while let Some(message) = self.messages.pop_front() {
            println!("Processing message: {:?}", message);
            let mut stream = message.connection.try_clone().unwrap();
            match message.command.command {
                RedisCommand::Ping => {
                    let response = "+PONG\r\n";
                    stream.write(response.as_bytes()).unwrap();
                }
                RedisCommand::Echo(content) => {
                    let response = format!("+{}\r\n", content);
                    stream.write(response.as_bytes()).unwrap();
                }
                RedisCommand::Set(key, value, expiry_flag, expiry_time) => {
                    // Insert the value, including expiry info if provided
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
                    // If this is the master replica
                    if self.role == Role::Master {
                        // Replicate to slaves
                        for (_port, slave) in &self.slaves {
                            let mut slave_stream = slave.try_clone().unwrap();
                            slave_stream.write(message.command.raw.as_bytes()).unwrap();
                        }
                    }
                    // Send OK response to client
                    let response = "+OK\r\n";
                    stream.write(response.as_bytes()).unwrap();
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
                                stream.write(response.as_bytes()).unwrap();
                            } else {
                                let response = format!("+{}\r\n", value.value);
                                stream.write(response.as_bytes()).unwrap();
                            }
                        }
                        None => {
                            let response = format!("+{}\r\n", value.value);
                            stream.write(response.as_bytes()).unwrap();
                        }
                    },
                    None => {
                        let response = "$-1\r\n";
                        stream.write(response.as_bytes()).unwrap();
                    }
                },
                RedisCommand::Command => {
                    let response = "+OK\r\n";
                    stream.write(response.as_bytes()).unwrap();
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
                    stream.write(response.as_bytes()).unwrap();
                }
                RedisCommand::ReplConf(key, value) => {
                    // When a stream has sent REPLCONF, we know it is a slave, add it to the slave list
                    if key == "listening-port" {
                        self.slaves.insert(value, stream.try_clone().unwrap());
                    }
                    let response = "+OK\r\n";
                    stream.write(response.as_bytes()).unwrap();
                }
                RedisCommand::Psync(_id, _offset) => {
                    let response = "+FULLRESYNC 8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb 0\r\n";
                    stream.write(response.as_bytes()).unwrap();
                    // Send the RDB file
                    let rdb = "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2";
                    let mut bytes = Vec::new();
                    for i in (0..rdb.len()).step_by(2) {
                        let byte = u8::from_str_radix(&rdb[i..i + 2], 16).unwrap();
                        bytes.push(byte);
                    }
                    let response = format!("${}\r\n", bytes.len());
                    stream.write(response.as_bytes()).unwrap();
                    stream.write(&bytes).unwrap();
                }
            }
        }
    }
    fn connect_to_master(&mut self) {
        let addr = self.master_addr.as_ref().unwrap();
        // Connect to the master
        match TcpStream::connect(addr) {
            Ok(stream) => {
                self.master_stream = Some(stream);
            }
            Err(e) => {
                panic!("Error connecting to master: {}", e);
            }
        }
        // Save a reference to the master stream
        let mut stream = self.master_stream.as_ref().unwrap();
        // Send PING to master to ensure connection
        println!("Sending PING to master...");
        let message = format!("*1\r\n$4\r\nPING\r\n");
        match stream.write(message.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                panic!("Error PINGing master: {}", e);
            }
        }
        // Read PONG from master
        self.expect_read("+PONG");
        // Send REPLCONF listening-port to master
        println!("Sending REPLCONF listening-port to master...");
        let message = format!(
            "*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n${}\r\n{}\r\n",
            self.port.to_string().len(),
            self.port
        );
        stream.write(message.as_bytes()).unwrap();
        // Read OK from master
        self.expect_read("+OK");
        // Send REPLCONF capa psync2 to master
        let message = format!("*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n");
        stream.write(message.as_bytes()).unwrap();
        // Read OK from master
        self.expect_read("+OK");
        // Send PSYNC to master
        let message = "*3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n";
        stream.write(message.as_bytes()).unwrap();
        // Read +FULLRESYNC from master
        let mut buf = [0; 1024];
        match stream.read(&mut buf) {
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
        // Read RDB from master
        match stream.read(&mut buf) {
            Ok(_) => {
                // TODO: Parse response and load RDB file into datastore
            }
            Err(e) => {
                panic!("Error reading from master: {}", e);
            }
        }
        // Handshake complete, mark the master stream as non-blocking
        stream.set_nonblocking(true).unwrap();
    }
    fn expect_read(&self, expected: &str) {
        let mut buf = [0; 1024];
        let mut stream = self.master_stream.as_ref().unwrap();
        match stream.read(&mut buf) {
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
