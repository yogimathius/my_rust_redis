use crate::{models::value::Value, replication::Role, server::RedisItem, server_legacy::Server};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub fn ping_handler(
    _: Arc<Mutex<HashMap<String, RedisItem>>>,
    _key: String,
    _: Vec<Value>,
) -> Option<Value> {
    Some(Value::SimpleString("PONG".to_string()))
}

pub fn echo_handler(
    _: Arc<Mutex<HashMap<String, RedisItem>>>,
    arg: String,
    _: Vec<Value>,
) -> Option<Value> {
    Some(Value::BulkString(arg))
}

pub async fn flushall_handler(
    cache: Arc<Mutex<HashMap<String, RedisItem>>>,
    _key: String,
    _: Vec<Value>,
) -> Option<Value> {
    let mut cache = cache.lock().await;
    cache.clear();
    Some(Value::SimpleString("OK".to_string()))
}

pub async fn info_handler(server: &mut Server) -> Option<Value> {
    let mut info = format!("role:{}", server.replication.role.to_string());
    match &server.replication.role {
        Role::Main => {
            info.push_str(&format!(
                "nmaster_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb"
            ));
            info.push_str("master_repl_offset:0");
        }
        Role::Slave => {
            info.push_str(&format!(
                "nmaster_host:{}nmaster_port:{}",
                server.replication.host, server.replication.port
            ));
        }
    };
    Some(Value::BulkString(info))
}
