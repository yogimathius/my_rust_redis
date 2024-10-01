use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    log,
    models::{redis_type::RedisType, value::Value},
    server::RedisItem,
};

pub async fn hvals_handler(
    cache: Arc<Mutex<HashMap<String, RedisItem>>>,
    key: String,
    _: Vec<Value>,
) -> Option<Value> {
    let mut cache = cache.lock().await;
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
