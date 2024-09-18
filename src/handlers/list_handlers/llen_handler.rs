use std::sync::MutexGuard;

use crate::{
    models::{redis_type::RedisType, value::Value},
    server::Server,
};

pub fn llen_handler(server: &mut Server, _key: String, args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    let key = match args.get(0) {
        Some(Value::BulkString(s)) => s.clone(),
        _ => {
            return Some(Value::Error(
                "ERR wrong number of arguments for 'lset' command".to_string(),
            ))
        }
    };

    let mut cache: MutexGuard<_> = server.cache.lock().unwrap();

    let item = match cache.get_mut(&key) {
        Some(item) => item,
        None => return Some(Value::Error("ERR no such key".to_string())),
    };
    if let RedisType::List = item.redis_type {
        if let Value::Array(ref list) = item.value {
            return Some(Value::Integer(list.len() as i64));
        } else {
            return Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string(),
            ));
        }
    }

    Some(Value::Error(
        "ERR operation against a key holding the wrong kind of value".to_string(),
    ))
}
