use crate::model::{Args, Value};
use crate::replica_client::ReplicaClient;
use crate::resp::RespHandler;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::net::TcpListener;

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    Main,
    Slave { host: String, port: u16 },
}

#[derive(Debug, PartialEq)]
pub struct RedisItem {
    value: String,
    created_at: Instant,
    expiration: Option<i64>,
}

#[derive(Clone)]
pub struct Server {
    pub cache: Arc<Mutex<HashMap<String, RedisItem>>>,
    role: Role,
    pub port: u16,
    pub sync: bool,
}

impl Server {
    pub fn new(args: Args) -> Self {
        let role = match args.replicaof {
            Some(vec) => {
                let mut iter = vec.into_iter();
                let addr = iter.next().unwrap();
                let port = iter.next().unwrap();
                Role::Slave {
                    host: addr,
                    port: port.parse::<u16>().unwrap(),
                }
            }
            None => Role::Main,
        };
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            role,
            port: args.port,
            sync: false,
        }
    }

    pub async fn match_replica(&mut self, args: Args) {
        match args.replicaof {
            Some(vec) => {
                let mut replica = ReplicaClient::new(vec).await.unwrap();

                replica.send_ping(&self).await.unwrap();

                while replica.handshakes < 4 {
                    match replica.read_response().await {
                        Ok(response) => {
                            replica.handshakes += 1;
                            replica.handle_response(&response, &self).await.unwrap();
                        }
                        Err(e) => {
                            eprintln!("Failed to read from stream: {}", e);
                        }
                    }
                }
            }
            None => {}
        }
    }

    pub async fn listen(&mut self, port: u16) {
        let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();
        println!("Listening on Port {}", port);

        loop {
            let stream = listener.accept().await;
            let server: Server = self.clone();
            match stream {
                Ok((stream, _)) => {
                    tokio::spawn(async move {
                        let mut handler = RespHandler::new(stream);
                        handler.handle_client(server).await.unwrap();
                    });
                }
                Err(e) => {
                    println!("error: {}", e);
                }
            }
        }
    }

    pub fn set(&mut self, args: Vec<Value>) -> Option<Value> {
        let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
        let value = unpack_bulk_str(args.get(1).unwrap().clone()).unwrap();
        let mut cache = self.cache.lock().unwrap();
        // add Expiration
        let expiration_time = match args.get(2) {
            None => None,
            Some(Value::BulkString(sub_command)) => {
                println!("sub_command = {:?} {}:?", sub_command, sub_command != "px");
                if sub_command != "px" {
                    panic!("Invalid expiration time")
                }
                match args.get(3) {
                    None => None,
                    Some(Value::BulkString(time)) => {
                        // add expiration
                        // parse time to i64
                        let time = time.parse::<i64>().unwrap();
                        Some(time)
                    }
                    _ => panic!("Invalid expiration time"),
                }
            }
            _ => panic!("Invalid expiration time"),
        };
        let redis_item = if let Some(exp_time) = expiration_time {
            RedisItem {
                value,
                created_at: Instant::now(),
                expiration: Some(exp_time),
            }
        } else {
            RedisItem {
                value,
                created_at: Instant::now(),
                expiration: None,
            }
        };
        cache.insert(key, redis_item);
        println!("Ok");
        Some(Value::SimpleString("OK".to_string()))
    }

    pub fn get(&mut self, args: Vec<Value>) -> Option<Value> {
        let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
        let cache = self.cache.lock().unwrap();
        match cache.get(&key) {
            Some(value) => {
                let response = if let Some(expiration) = value.expiration {
                    let now = Instant::now();
                    if now.duration_since(value.created_at).as_millis() > expiration as u128 {
                        Value::NullBulkString
                    } else {
                        Value::BulkString(value.value.clone())
                    }
                } else {
                    Value::BulkString(value.value.clone())
                };
                Some(response)
            }
            None => Some(Value::NullBulkString),
        }
    }

    pub fn info(&self) -> Option<Value> {
        let mut info = format!("role:{}", self.role.to_string());
        match &self.role {
            Role::Main => {
                info.push_str(&format!(
                    "nmaster_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb"
                ));
                info.push_str("master_repl_offset:0");
            }
            Role::Slave { host, port } => {
                info.push_str(&format!("nmaster_host:{}nmaster_port:{}", host, port));
            }
        };
        Some(Value::BulkString(info))
    }

    pub fn ping(&self) -> Option<Value> {
        match &self.role {
            Role::Main => None,
            Role::Slave { host: _, port: _ } => {
                let msg = vec![Value::BulkString(String::from("PING"))];
                let payload = Value::Array(msg);
                Some(payload)
            }
        }
    }

    pub fn psync(&self) -> Option<Value> {
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

    pub fn sync(&mut self) -> Option<Value> {
        match &self.role {
            Role::Main => None,
            Role::Slave { host: _, port: _ } => {
                self.sync = true;
                let msg = vec![Value::BulkString(String::from("SYNC"))];
                let payload = Value::Array(msg);
                Some(payload)
            }
        }
    }

    pub fn generate_replconf(&self, command: &str, params: Vec<(&str, String)>) -> Option<Value> {
        match &self.role {
            Role::Main => None,
            Role::Slave { host: _, port: _ } => {
                let mut msg = vec![Value::BulkString(command.to_string())];
                for (key, value) in params {
                    msg.push(Value::BulkString(key.to_string()));
                    msg.push(Value::BulkString(value.to_string()));
                }
                let payload = Value::Array(msg);
                Some(payload)
            }
        }
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

pub fn extract_command(value: Value) -> Result<(String, Vec<Value>)> {
    match value {
        Value::Array(a) => Ok((
            unpack_bulk_str(a.first().unwrap().clone())?,
            a.into_iter().skip(1).collect(),
        )),
        _ => Err(anyhow::anyhow!("Unexpected command format")),
    }
}

fn unpack_bulk_str(value: Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Expected bulk string")),
    }
}
