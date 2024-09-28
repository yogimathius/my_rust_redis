use crate::{
    log,
    models::{redis_type::RedisType, value::Value},
    server::Server,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn hvals_handler(
    server: Arc<Mutex<Server>>,
    key: String,
    _: Vec<Value>,
) -> Option<Value> {
    let server = server.lock().await;

    let mut cache = server.cache.lock().await;
    match cache.get_mut(&key) {
        Some(item) => {
            log!("Item: {:?}", item);
            if let RedisType::Hash = item.redis_type {
                if let Value::Hash(hash) = &item.value {
                    let mut values: Vec<_> = hash.values().cloned().collect();
                    values.sort_by(|a, b| a.clone().serialize().cmp(&b.clone().serialize())); // Custom comparison
                    Some(Value::Array(values))
                } else {
                    Some(Value::Error(
                        "ERR operation against a key holding the wrong kind of value".to_string(),
                    ))
                }
            } else {
                Some(Value::Error(
                    "ERR operation against a key holding the wrong kind of value".to_string(),
                ))
            }
        }
        None => Some(Value::Array(vec![])),
    }
}
