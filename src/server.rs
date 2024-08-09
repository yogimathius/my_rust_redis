use crate::resp::{self, Value};
use crate::Args;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    Main,
    Replica { host: String, port: u16 },
}

#[derive(Debug, PartialEq)]
pub struct RedisItem {
    value: String,
    created_at: Instant,
    expiration: Option<i64>,
}

#[derive(Clone)]
pub struct Server {
    cache: Arc<Mutex<HashMap<String, RedisItem>>>,
    role: Role,
}

impl Server {
    pub fn new(args: Args) -> Self {
        let role = match args.replicaof {
            Some(vec) => {
                let mut iter = vec.into_iter();
                let addr = iter.next().unwrap();
                let port = iter.next().unwrap();
                Role::Replica {
                    host: addr,
                    port: port.parse::<u16>().unwrap(),
                }
            }
            None => Role::Main,
        };
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            role,
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
                        handle_client(stream, server).await;
                    });
                }
                Err(e) => {
                    println!("error: {}", e);
                }
            }
        }
    }

    pub fn set(&mut self, args: Vec<Value>) -> Value {
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

        Value::SimpleString("OK".to_string())
    }

    pub fn get(&self, args: Vec<Value>) -> Value {
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
                response
            }
            None => Value::NullBulkString,
        }
    }

    pub fn info(&self) -> Value {
        let mut info = format!("role:{}", self.role.to_string());
        match &self.role {
            Role::Main => {
                info.push_str(&format!(
                    "nmaster_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb"
                ));
                info.push_str("master_repl_offset:0");
            }
            Role::Replica { host, port } => {
                info.push_str(&format!("nmaster_host:{}nmaster_port:{}", host, port));
            }
        };
        Value::BulkString(info)
    }

    pub fn ping(&self) -> Option<Value> {
        match &self.role {
            Role::Main => None,
            Role::Replica { host: _, port: _ } => {
                let msg = vec![Value::BulkString(String::from("ping"))];
                let payload = Value::Array(msg);
                Some(payload)
            }
        }
    }

    pub fn replconf(&self) -> Option<Value> {
        match &self.role {
            Role::Main => None,
            Role::Replica { host: _, port: _ } => {
                let msg = vec![
                    Value::BulkString(String::from("REPLCONF")),
                    Value::BulkString(String::from("listening-port")),
                    Value::BulkString(String::from("6380")),
                ];
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
            Self::Replica { host: _, port: _ } => String::from("Replica"),
        }
    }
}

async fn handle_client(stream: TcpStream, mut server: Server) {
    let mut handler = resp::RespHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();

        let response = if let Some(value) = value {
            let (command, args) = extract_command(value).unwrap();

            match command.as_str() {
                "ping" => Value::SimpleString("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                "get" => server.get(args),
                "set" => server.set(args),
                "INFO" => server.info(),
                "REPLCONF" => Value::SimpleString("OK".to_string()),
                _ => panic!("Cannot handle command {}", command),
            }
        } else {
            break;
        };

        handler.write_value(response).await.unwrap();
    }
}

fn extract_command(value: Value) -> Result<(String, Vec<Value>)> {
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
