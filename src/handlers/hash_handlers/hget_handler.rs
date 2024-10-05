use std::collections::HashMap;

use crate::models::{redis_item::RedisItem, redis_type::RedisType, value::Value};

pub async fn hget_handler(
    cache: HashMap<String, RedisItem>,
    key: String,
    args: Vec<Value>,
) -> Option<Value> {
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
