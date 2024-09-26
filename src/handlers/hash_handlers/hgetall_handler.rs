use crate::{
    log,
    models::{redis_type::RedisType, value::Value},
    server::Server,
};

pub fn hgetall_handler(server: &mut Server, key: String, _: Vec<Value>) -> Option<Value> {
    let mut cache = server.cache.lock().unwrap();
    match cache.get_mut(&key) {
        Some(item) => {
            log!("Item: {:?}", item);
            if let RedisType::Hash = item.redis_type {
                if let Value::Hash(hash) = &item.value {
                    let mut sorted_keys: Vec<_> = hash.keys().cloned().collect();
                    sorted_keys.sort();

                    let hash_arr: Vec<Value> = sorted_keys
                        .into_iter()
                        .flat_map(|k| {
                            let v = hash.get(&k).unwrap();
                            vec![Value::BulkString(k), v.clone()]
                        })
                        .collect();

                    Some(Value::Array(hash_arr))
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
