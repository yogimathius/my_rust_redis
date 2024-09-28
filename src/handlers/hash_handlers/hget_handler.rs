use crate::{
    models::{redis_type::RedisType, value::Value},
    server::Server,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn hget_handler(
    server: Arc<Mutex<Server>>,
    key: String,
    args: Vec<Value>,
) -> Option<Value> {
    let server = server.lock().await;

    let cache = server.cache.lock().await;
    match cache.get(&key) {
        Some(item) if item.redis_type == RedisType::Hash => match &item.value {
            Value::Hash(hash) => match args.get(0) {
                Some(Value::BulkString(field)) => hash.get(field).cloned(),
                _ => Some(Value::Error(
                    "ERR arguments must contain a value for every field".to_string(),
                )),
            },
            _ => Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string(),
            )),
        },
        Some(_) => Some(Value::Error(
            "ERR operation against a key holding the wrong kind of value".to_string(),
        )),
        None => None,
    }
}
