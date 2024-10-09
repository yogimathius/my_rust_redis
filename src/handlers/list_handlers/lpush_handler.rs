use std::time::Instant;

use crate::{
    models::{redis_type::RedisType, value::Value},
    server::{RedisItem, Server},
};

pub fn lpush_handler(server: &mut Server, key: String, args: Vec<Value>) -> Option<Value> {
    let new_item = match args.get(0) {
        Some(Value::BulkString(v)) => v.clone(),
        _ => return Some(Value::Error("ERR value is not a bulk string".to_string())),
    };

    let mut cache = server.cache.lock().unwrap();
    match cache.get_mut(&key) {
        Some(item) => {
            if let Value::Array(ref mut list) = item.value {
                list.insert(0, Value::BulkString(new_item));
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
                created_at: Instant::now().elapsed().as_secs() as i64,
                redis_type: RedisType::List,
            };

            cache.insert(key.clone(), redis_item);
            Some(Value::Integer(1))
        }
    }
}
