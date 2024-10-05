use crate::models::value::Value;
use crate::models::{redis_item::RedisItem, role::Role};
use crate::server::Server;

use std::collections::HashMap;

pub fn ping_handler(_: HashMap<String, RedisItem>, _key: String, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("PONG".to_string()))
}

pub fn echo_handler(_: HashMap<String, RedisItem>, arg: String, _: Vec<Value>) -> Option<Value> {
    Some(Value::BulkString(arg))
}

pub async fn flushall_handler(
    mut cache: HashMap<String, RedisItem>,
    _key: String,
    _: Vec<Value>,
) -> Option<Value> {
    cache.clear();
    Some(Value::SimpleString("OK".to_string()))
}

pub async fn info_handler(server: &mut Server) -> Option<Value> {
    let mut info = format!("role:{}", server.role.to_string());
    match &server.role {
        Role::Master => {
            info.push_str(&format!(
                "nmaster_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb"
            ));
            info.push_str("master_repl_offset:0");
        }
        _ => {}
    };
    Some(Value::BulkString(info))
}
