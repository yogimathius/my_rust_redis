use crate::resp::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use anyhow::Result;

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    Master,
    Slave { host: String, port: u16 }
}

#[derive(Debug, PartialEq)]
struct Entry {
    value: String,
    expiry: Option<Instant>,
}
impl Entry {
    fn new(value: String, expiry: Option<Instant>) -> Self {
        Self { value, expiry }
    }
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
    pub fn new(role: Role) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            role,
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
        let mut cache = self.cache.lock().unwrap();
        match cache.get(&key) {
            Some(value) => {
                let response = if let Some(expiration) = value.expiration {
                    let now = Instant::now();
                    if now.duration_since(value.created_at).as_millis()
                        > expiration as u128
                    {
                        Value::NullBulkString
                    } else {
                        Value::BulkString(value.value.clone())
                    }
                } else {
                    Value::BulkString(value.value.clone())
                };
                response
            },
            None => Value::NullBulkString,
        }
    }

    pub fn info(&self) -> String {
        let mut info = format!("role:{}", self.role.to_string());
        match self.role {
            Role::Master => {
                info.push_str(&format!("nmaster_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb"));
                info.push_str("master_repl_offset:0");
            }
            Role::Slave { host, port } => {
                info.push_str(&format!("nmaster_host:{}nmaster_port:{}", host, port));
            }
        };
        info
    }

    pub fn ping(&self) -> Option<Value> {
        match self.role {
            Role::Master => None,
            Role::Slave { host, port } => {
                let msg = Value::BulkString(String::from("ping"));
                Some(msg)
            }
        }
    }
}


impl ToString for Role {
    fn to_string(&self) -> String {
        match self {
            Self::Master => String::from("master"),
            Self::Slave { host, port } => String::from("slave"),
        }
    }
}

fn unpack_bulk_str(value: Value) ->  Result<String>{
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Expected bulk string")),

    }
}