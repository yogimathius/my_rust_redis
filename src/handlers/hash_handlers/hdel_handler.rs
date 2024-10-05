use std::collections::HashMap;

use crate::models::{redis_item::RedisItem, redis_type::RedisType, value::Value};

pub async fn hdel_handler(
    mut cache: HashMap<String, RedisItem>,
    key: String,
    args: Vec<Value>,
) -> Option<Value> {
    if args.is_empty() {
        return Some(Value::Integer(0));
    }

    match cache.get_mut(&key) {
        Some(item) => {
            if let RedisType::Hash = item.redis_type {
                let mut count = 0;
                if let Value::Hash(ref mut hash) = item.value {
                    for field in args {
                        if let Value::BulkString(field) = field {
                            if hash.remove(&field).is_some() {
                                count += 1;
                            }
                        } else {
                            return Some(Value::Error(
                                "ERR arguments must contain a value for every field".to_string(),
                            ));
                        }
                    }
                    return Some(Value::Integer(count));
                }
                Some(Value::Error(
                    "ERR operation against a key holding the wrong kind of value".to_string(),
                ))
            } else {
                Some(Value::Error(
                    "ERR operation against a key holding the wrong kind of value".to_string(),
                ))
            }
        }
        None => Some(Value::Integer(0)),
    }
}
