use std::collections::HashMap;

use crate::{
    log,
    models::{redis_item::RedisItem, redis_type::RedisType, value::Value},
};

pub async fn hlen_handler(
    mut cache: HashMap<String, RedisItem>,
    key: String,
    _: Vec<Value>,
) -> Option<Value> {
    match cache.get_mut(&key) {
        Some(item) => {
            log!("Item: {:?}", item);
            if let RedisType::Hash = item.redis_type {
                if let Value::Hash(hash) = &item.value {
                    Some(Value::Integer(hash.len() as i64))
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
        None => Some(Value::Integer(0)),
    }
}
