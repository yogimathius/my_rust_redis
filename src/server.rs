use crate::log;
use crate::models::args::Args;
use crate::models::redis_type::RedisType;
use crate::models::value::Value;
use crate::replica::ReplicaClient;
use crate::resp::RespHandler;
use crate::utilities::ServerState;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::time::{interval, sleep, Duration};

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    Main,
    Slave { host: String, port: u16 },
}

#[derive(Debug, PartialEq)]
pub struct RedisItem {
    pub value: Value,
    pub created_at: Instant,
    pub expiration: Option<i64>,
    pub redis_type: RedisType,
}

#[derive(Clone, Debug)]
pub struct Server {
    pub cache: Arc<Mutex<HashMap<String, RedisItem>>>,
    pub role: Role,
    pub port: u16,
    pub sync: bool,
    pub server_state: ServerState,
}

impl Server {
    pub fn new(args: Args) -> Self {
        let role = match args.replicaof {
            Some(vec) => {
                let mut iter = vec.into_iter();
                let addr = iter.next().unwrap();
                let _ = iter.next().unwrap();
                Role::Slave {
                    host: addr,
                    port: args.port,
                }
            }
            None => Role::Main,
        };
        let mut server = Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            role,
            port: args.port,
            sync: false,
            server_state: ServerState::Initialising,
        };

        // Read RDB dump
        if let Err(e) = server.read_rdb_dump() {
            log!("Error reading RDB dump: {:?}", e);
        }

        server
    }

    fn read_rdb_dump(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new("dump.rdb");
        if !path.exists() {
            log!("RDB file does not exist. Creating a new one.");
            self.create_empty_rdb(path)?;
            return Ok(());
        }

        let mut file = File::open(path)?;
        let file_size = file.metadata()?.len();
        log!("RDB file size: {} bytes", file_size);

        if file_size == 0 {
            log!("RDB file is empty. Initializing with empty cache.");
            return Ok(());
        }

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        log!("Read {} bytes from RDB file", buffer.len());

        if buffer.len() < 9 {
            log!("RDB file is too short. Initializing with empty cache.");
            return Ok(());
        }

        if &buffer[0..9] != b"REDIS0009" {
            log!("Invalid RDB file header: {:?}", &buffer[0..9]);
            return Err("Invalid RDB file format".into());
        }

        let mut index = 9;
        let mut cache = self.cache.lock().unwrap();

        while index < buffer.len() {
            log!("Processing byte at index {}: {:02X}", index, buffer[index]);
            match buffer[index] {
                0xFF => {
                    log!("Reached end of RDB file");
                    break;
                }
                0xFE => {
                    log!("Resizedb marker found, skipping");
                    index += 1;
                }
                _ => {
                    if index + 1 >= buffer.len() {
                        log!("Unexpected end of file");
                        break;
                    }
                    let key_len = buffer[index] as usize;
                    index += 1;
                    if index + key_len > buffer.len() {
                        log!("Key length exceeds file size");
                        break;
                    }
                    let key = String::from_utf8_lossy(&buffer[index..index + key_len]).to_string();
                    index += key_len;

                    if index >= buffer.len() {
                        log!("Unexpected end of file after key");
                        break;
                    }
                    let value_type = buffer[index];
                    index += 1;

                    let value = match value_type {
                        0 => {
                            if index >= buffer.len() {
                                log!("Unexpected end of file before value length");
                                break;
                            }
                            let value_len = buffer[index] as usize;
                            index += 1;
                            if index + value_len > buffer.len() {
                                log!("Value length exceeds file size");
                                break;
                            }
                            let value = String::from_utf8_lossy(&buffer[index..index + value_len])
                                .to_string();
                            index += value_len;
                            Value::BulkString(value)
                        }
                        _ => {
                            log!("Unsupported value type: {}", value_type);
                            return Err(format!("Unsupported value type: {}", value_type).into());
                        }
                    };

                    let item = RedisItem {
                        value,
                        created_at: std::time::Instant::now(),
                        expiration: None,
                        redis_type: RedisType::String,
                    };

                    cache.insert(key, item);
                }
            }
        }

        log!("RDB dump loaded successfully");
        Ok(())
    }

    fn create_empty_rdb(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(path)?;
        file.write_all(b"REDIS0009\0")?;
        file.write_all(&[0xFF])?;
        Ok(())
    }

    fn dump_rdb(
        cache: &Arc<Mutex<HashMap<String, RedisItem>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new("dump.rdb");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        file.write_all(b"REDIS0009\0")?; // Redis RDB version identifier

        let cache_guard = cache.lock().unwrap();
        for (key, item) in cache_guard.iter() {
            // Write key
            file.write_all(&[key.len() as u8])?;
            file.write_all(key.as_bytes())?;

            // Write value type
            file.write_all(&[0])?; // Assuming all values are strings for now

            // Write value
            if let Value::BulkString(value) = &item.value {
                file.write_all(&[value.len() as u8])?;
                file.write_all(value.as_bytes())?;
            } else {
                return Err("Unsupported value type".into());
            }
        }

        file.write_all(&[0xFF])?; // End of RDB marker
        Ok(())
    }

    pub async fn match_replica(&mut self, args: Args) {
        match args.replicaof {
            Some(vec) => {
                let mut replica = ReplicaClient::new(vec).await.unwrap();
                replica.send_ping(&self).await.unwrap();

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
            None => {}
        }
    }

    pub async fn listen(&mut self, port: u16) {
        let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();
        log!("Listening on Port {}", port);

        // Start periodic RDB dump job
        let cache_clone = self.cache.clone();
        tokio::spawn(async move {
            // Wait for 6 minutes before the first dump
            sleep(Duration::from_secs(360)).await;

            let mut interval = interval(Duration::from_secs(360)); // 6 minutes
            loop {
                if let Err(e) = Self::dump_rdb(&cache_clone) {
                    log!("Error dumping RDB: {:?}", e);
                } else {
                    log!("RDB dump completed successfully");
                }
                interval.tick().await;
            }
        });

        loop {
            let stream = listener.accept().await;
            let server: Server = self.clone();
            match stream {
                Ok((stream, _)) => {
                    tokio::spawn(async move {
                        let mut handler = RespHandler::new(stream);
                        match handler.handle_client(server).await {
                            Ok(_) => log!("Client disconnected gracefully"),
                            Err(e) => log!("Client disconnected with error: {}", e),
                        }
                    });
                }
                Err(e) => {
                    log!("Error accepting connection: {}", e);
                }
            }
        }
    }

    pub fn send_ping(&self) -> Option<Value> {
        match &self.role {
            Role::Main => None,
            Role::Slave { host: _, port: _ } => {
                let msg = vec![Value::BulkString(String::from("PING"))];
                let payload = Value::Array(msg);
                Some(payload)
            }
        }
    }

    pub fn send_psync(&self) -> Option<Value> {
        log!("Syncing with master");
        log!("self.role {:?}", self.role);

        match &self.role {
            Role::Main => None,
            Role::Slave { host: _, port: _ } => {
                let msg = vec![
                    Value::BulkString(String::from("PSYNC")),
                    Value::BulkString(String::from("?")),
                    Value::BulkString(String::from("-1")),
                ];
                let payload = Value::Array(msg);
                Some(payload)
            }
        }
    }

    pub fn generate_replconf(&self, command: &str, params: Vec<(&str, String)>) -> Option<Value> {
        let mut msg = vec![Value::BulkString(command.to_string())];
        for (key, value) in params {
            msg.push(Value::BulkString(key.to_string()));
            msg.push(Value::BulkString(value.to_string()));
        }
        let payload = Value::Array(msg);
        Some(payload)
    }
}

impl ToString for Role {
    fn to_string(&self) -> String {
        match self {
            Self::Main => String::from("master"),
            Self::Slave { host: _, port: _ } => String::from("slave"),
        }
    }
}
