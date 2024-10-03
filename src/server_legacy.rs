use std::{
    collections::{HashMap, VecDeque},
    io::{Read, Write},
    sync::Arc,
};

use anyhow::Error;
use regex::Regex;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use crate::{
    command::Command,
    commands::COMMAND_HANDLERS,
    connection::Connection,
    handlers::{info_handler, set_handler},
    log,
    models::{message::Message, redis_item::RedisItem, role::Role, value::Value},
    utilities::extract_command,
};

pub struct Server {
    port: u16,
    role: Role,
    master_addr: Option<String>,
    master_stream: Option<Connection>,
    master_replid: String,
    master_repl_offset: u32,
    listener: Option<TcpListener>,
    connections: Vec<TcpStream>,
    connections_v2: Vec<Connection>,
    messages: VecDeque<Message>,
    slaves: HashMap<String, TcpStream>,
    pub cache: Arc<Mutex<HashMap<String, RedisItem>>>,
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
            connections_v2: Vec::new(),
            messages: VecDeque::new(),
            slaves: HashMap::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn run(&mut self) {
        // If this replica is a slave, connect to the master
        if self.role == Role::Slave {
            self.connect_to_master();
        }
        // Start listening for connections
        match TcpListener::bind(format!("127.0.0.1:{}", self.port)).await {
            Ok(listener) => {
                self.listener = Some(listener);
            }
            Err(e) => {
                panic!("Error binding to port: {}", e);
            }
        }
        println!("Listening on 127.0.0.1:{}...", self.port);
    }

    pub async fn accept_connections(&mut self) {
        let listener = self.listener.as_ref().unwrap();
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    // self.connections.push(stream);
                    let conn = Connection::new(None, Some(stream)).await;
                    self.connections_v2.push(conn);
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

    pub async fn read_messages(&mut self) {
        let mut closed_indices = Vec::new();
        for (index, conn) in self.connections_v2.iter_mut().enumerate() {
            let (should_close, new_commands) = Server::read_messages_from(conn).await;
            if should_close {
                closed_indices.push(index);
            }
            self.messages
                .extend(new_commands.into_iter().map(|command| Message {
                    connection: conn.clone(),
                    command,
                }));
        }
        // Remove closed connections
        for &index in closed_indices.iter().rev() {
            self.connections.remove(index);
        }
        // Read from the master stream if it exists
        if self.master_stream.is_some() {
            let (_should_close, new_commands) =
                Server::read_messages_from(self.master_stream.as_mut().unwrap()).await;
            if new_commands.len() > 0 {
                println!("Received {} commands from master", new_commands.len());
                self.messages
                    .extend(new_commands.into_iter().map(|command| Message {
                        connection: self.master_stream.as_mut().unwrap().clone(),
                        command,
                    }));
            }
        }
    }

    async fn read_messages_from(conn: &mut Connection) -> (bool, Vec<Command>) {
        let mut buf = [0; 1024];
        let mut new_commands: Vec<Command> = Vec::new();
        // Read from the connection until there are no more messages
        let mut should_close = false;
        loop {
            match conn.stream.read(&mut buf).await {
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

    async fn connect_to_master(&mut self) {
        let addr = self.master_addr.as_ref().unwrap();
        // Connect to the master
        match TcpStream::connect(addr).await {
            Ok(stream) => {
                let stream = Connection::new(None, Some(stream)).await;
                self.master_stream = Some(stream);
            }
            Err(e) => {
                panic!("Error connecting to master: {}", e);
            }
        }

        println!("Sending PING to master...");
        self.master_stream
            .as_mut()
            .unwrap()
            .write_value(Value::SimpleString("PING".to_string()))
            .await
            .unwrap();
        // Read PONG from master
        self.master_stream.as_mut().unwrap().expect_read("+PONG");
        // Send REPLCONF listening-port to master
        println!("Sending REPLCONF listening-port to master...");
        self.send_replconf_two().await.unwrap();
        // Read OK from master
        self.master_stream.as_mut().unwrap().expect_read("+OK");
        // Send REPLCONF capa psync2 to master
        let message = format!("*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n");
        self.master_stream
            .as_mut()
            .unwrap()
            .stream
            .write(message.as_bytes())
            .await;
        // Read OK from master
        self.master_stream.as_mut().unwrap().expect_read("+OK");
        // Send PSYNC to master
        let message = "*3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n";
        self.master_stream
            .as_mut()
            .unwrap()
            .stream
            .write(message.as_bytes())
            .await;
        // Read +FULLRESYNC from master
        let mut buf = [0; 1024];
        match self
            .master_stream
            .as_mut()
            .unwrap()
            .stream
            .read(&mut buf)
            .await
        {
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
        } // Read RDB from master
        match self
            .master_stream
            .as_mut()
            .unwrap()
            .stream
            .read(&mut buf)
            .await
        {
            Ok(_) => {
                // TODO: Parse response and load RDB file into datastore
            }
            Err(e) => {
                panic!("Error reading from master: {}", e);
            }
        }
        // Handshake complete, mark the master stream as non-blocking
    }

    pub async fn handshake(&mut self, mut conn: Connection) -> Result<(), Error> {
        log!("Starting handshake");
        conn.write_value(Value::SimpleString("PING".to_string()))
            .await?;

        let _response = conn.read_value().await?;

        self.send_replconf_two().await?;

        let _response = conn.read_value().await?;

        self.send_replconf().await?;

        let _response = conn.read_value().await?;

        let psync = Value::Array(vec![
            Value::BulkString("PSYNC".to_string()),
            Value::BulkString("?".to_string()),
            Value::BulkString("-1".to_string()),
        ]);

        conn.write_value(psync).await?;

        let response: Option<Value> = conn.read_value().await?;
        log!("Handshake response: {:?}", response);
        Ok(())
    }

    pub async fn execute_command(
        &mut self,
        cache: Arc<Mutex<HashMap<String, RedisItem>>>,
        value: Value,
        conn: &mut Connection,
    ) -> Result<(), Error> {
        let (command, key, args) = extract_command(value.clone()).unwrap();
        log!("executing command: {:?}", command);
        match command.as_str() {
            "PING" => {
                let value = Value::SimpleString("PONG".to_string());
                conn.write_value(value).await.unwrap();
                Ok(())
            }
            "REPLCONF" => {
                log!("REPLCONF command");
                let value = Value::SimpleString("OK".to_string());
                conn.write_value(value).await.unwrap();
                Ok(())
            }
            "PSYNC" => {
                let value = Value::SimpleString(format!("FULLRESYNC {} 0", self.master_replid));
                conn.write_value(value).await.unwrap();
                self.write_rdb(conn).await.unwrap();
                let mut conn_clone = conn.clone();

                log!("Done sending values");
                Ok(())
            }
            "SET" => {
                log!("SET command");
                set_handler(cache, key, args).await;
                Ok(())
            }
            "INFO" => {
                let response = info_handler(self).await;
                conn.write_value(response.unwrap()).await.unwrap();
                Ok(())
            }
            _ => {
                if let Some(command_function) = COMMAND_HANDLERS.get(command.as_str()) {
                    command_function.handle(cache, key, args).await;
                    Ok(())
                } else {
                    let value = Value::Error("ERR unknown command".to_string());
                    conn.write_value(value).await.unwrap();
                    Err(anyhow::Error::msg("Unknown command"))
                }
            }
        }
    }

    pub async fn write_rdb(&mut self, conn: &mut Connection) -> Result<(), Error> {
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
        log!("Writing RDB file");
        log!("header: {:?}", header);
        conn.write_all(header.as_bytes()).await?;
        log!("contents: {:?}", contents);
        conn.write_all(&contents).await?;
        log!("Wrote RDB file");
        Ok(())
    }

    pub async fn send_replconf(&mut self) -> Result<(), Error> {
        let command = "REPLCONF";
        let msg = Value::Array(vec![
            Value::BulkString(String::from(command)),
            Value::BulkString(String::from("listening-port")),
            Value::BulkString(self.port.to_string()),
        ]);
        self.master_stream
            .as_mut()
            .unwrap()
            .write_value(msg)
            .await?;
        Ok(())
    }

    pub async fn send_replconf_two(&mut self) -> Result<(), Error> {
        let command = "REPLCONF";
        let msg = Value::Array(vec![
            Value::BulkString(String::from(command)),
            Value::BulkString(String::from("capa")),
            Value::BulkString(String::from("psync2")),
        ]);
        self.master_stream
            .as_mut()
            .unwrap()
            .write_value(msg)
            .await?;
        Ok(())
    }
}
