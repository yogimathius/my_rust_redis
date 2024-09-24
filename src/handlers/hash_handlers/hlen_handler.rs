use crate::{
    models::{redis_type::RedisType, value::Value},
    server::Server,
};

pub fn hlen_handler(server: &mut Server, key: String, _: Vec<Value>) -> Option<Value> {
    let mut cache = server.cache.lock().unwrap();
    println!("Cache: {:?}", cache);
    println!("Key: {:?}", key);
    match cache.get_mut(&key) {
        Some(item) => {
            println!("Item: {:?}", item);
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
