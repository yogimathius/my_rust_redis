use crate::{
    models::value::Value,
    server::{Role, Server},
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn ping_handler(_: Arc<Mutex<Server>>, _key: String, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("PONG".to_string()))
}

pub fn echo_handler(_: Arc<Mutex<Server>>, arg: String, _: Vec<Value>) -> Option<Value> {
    Some(Value::BulkString(arg))
}

pub async fn flushall_handler(
    server: Arc<Mutex<Server>>,
    _key: String,
    _: Vec<Value>,
) -> Option<Value> {
    let server = server.lock().await;
    let mut cache = server.cache.lock().await;
    cache.clear();
    Some(Value::SimpleString("OK".to_string()))
}

pub async fn info_handler(server: Arc<Mutex<Server>>, _: String, _: Vec<Value>) -> Option<Value> {
    let server = server.lock().await;
    let mut info = format!("role:{}", server.role.to_string());
    match &server.role {
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
