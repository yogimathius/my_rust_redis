use crate::{
    log,
    models::{redis_type::RedisType, value::Value},
    server::Server,
};

pub fn hkeys_handler(server: &mut Server, key: String, _: Vec<Value>) -> Option<Value> {
    let mut cache = server.cache.lock().unwrap();
    match cache.get_mut(&key) {
        Some(item) => {
            log!("Item: {:?}", item);
            if let RedisType::Hash = item.redis_type {
                if let Value::Hash(hash) = &item.value {
                    let mut keys: Vec<_> = hash.keys().cloned().collect();
                    keys.sort();
                    let keys = keys.into_iter().map(Value::BulkString).collect();
                    Some(Value::Array(keys))
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
