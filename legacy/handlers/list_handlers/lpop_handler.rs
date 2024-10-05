use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::models::{redis_item::RedisItem, value::Value};
pub async fn lpop_handler(
    cache: Arc<Mutex<HashMap<String, RedisItem>>>,
    key: String,
    _args: Vec<Value>,
) -> Option<Value> {
    let mut cache = cache.lock().await;
    match cache.get_mut(&key) {
        Some(item) => {
            if let Value::Array(ref mut list) = item.value {
                if list.is_empty() {
                    Some(Value::NullBulkString)
                } else {
                    Some(list.remove(0))
                }
            } else {
                Some(Value::Error(
                    "ERR operation against a key holding the wrong kind of value".to_string(),
                ))
            }
        }
        None => Some(Value::NullBulkString),
    }
}
