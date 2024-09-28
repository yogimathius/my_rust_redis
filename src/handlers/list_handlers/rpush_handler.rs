use crate::{
    models::{redis_type::RedisType, value::Value},
    server::{RedisItem, Server},
};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

pub async fn rpush_handler(
    server: Arc<Mutex<Server>>,
    key: String,
    args: Vec<Value>,
) -> Option<Value> {
    let server = server.lock().await;

    let new_item = match args.get(0) {
        Some(Value::BulkString(v)) => v.clone(),
        _ => return Some(Value::Error("ERR value is not a bulk string".to_string())),
    };

    let mut cache = server.cache.lock().await;

    match cache.get_mut(&key) {
        Some(item) => {
            if let Value::Array(ref mut list) = item.value {
                list.push(Value::BulkString(new_item));
                Some(Value::Integer(list.len() as i64))
            } else {
                Some(Value::Error(
                    "ERR operation against a key holding the wrong kind of value".to_string(),
                ))
            }
        }
        None => {
            let list = vec![Value::BulkString(new_item)];
            let redis_item = RedisItem {
                value: Value::Array(list),
                expiration: None,
                created_at: Instant::now(),
                redis_type: RedisType::List,
            };

            cache.insert(key.clone(), redis_item);
            Some(Value::Integer(1))
        }
    }
}
