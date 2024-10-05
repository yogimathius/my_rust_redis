use std::collections::HashMap;

use crate::models::{redis_item::RedisItem, value::Value};

pub async fn rpop_handler(
    mut cache: HashMap<String, RedisItem>,
    key: String,
    _args: Vec<Value>,
) -> Option<Value> {
    match cache.get_mut(&key) {
        Some(item) => {
            if let Value::Array(ref mut list) = item.value {
                if list.is_empty() {
                    Some(Value::NullBulkString)
                } else {
                    Some(list.remove(list.len() - 1))
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
