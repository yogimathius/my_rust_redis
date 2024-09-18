use crate::{
    models::{redis_type::RedisType, value::Value},
    server::Server,
};
use std::sync::MutexGuard;

pub fn lset_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    let key = match args.get(0) {
        Some(Value::BulkString(s)) => s.clone(),
        _ => {
            return Some(Value::Error(
                "ERR wrong number of arguments for 'lset' command".to_string(),
            ))
        }
    };

    let index = match args.get(1) {
        Some(Value::Integer(i)) => *i as usize,
        _ => return Some(Value::Error("ERR index is not an integer".to_string())),
    };

    let new_value = match args.get(2) {
        Some(v) => v.clone(),
        _ => {
            return Some(Value::Error(
                "ERR wrong number of arguments for 'lset' command".to_string(),
            ))
        }
    };

    // Lock the cache
    let mut cache: MutexGuard<_> = server.cache.lock().unwrap();

    // Retrieve the list associated with the key
    let item = match cache.get_mut(&key) {
        Some(item) => item,
        None => return Some(Value::Error("ERR no such key".to_string())),
    };

    if let RedisType::List = item.redis_type {
        if let Value::Array(ref mut list) = item.value {
            if index < list.len() {
                list[index] = new_value;
                return Some(Value::SimpleString("OK".to_string()));
            } else {
                return Some(Value::Error("ERR index out of range".to_string()));
            }
        }
    }

    Some(Value::Error(
        "ERR operation against a key holding the wrong kind of value".to_string(),
    ))
}
