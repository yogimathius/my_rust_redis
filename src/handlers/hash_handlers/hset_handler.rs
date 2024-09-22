use crate::{
    models::{redis_type::RedisType, value::Value},
    server::Server,
};

pub fn hset_handler(server: &mut Server, key: String, args: Vec<Value>) -> Option<Value> {
    for chunk in args.chunks(2) {
        match chunk {
            [Value::BulkString(_), Value::BulkString(_)] => continue,
            [Value::BulkString(_), Value::Integer(_)] => continue,
            [Value::BulkString(_), Value::Array(_)] => continue,
            _ => {
                return Some(Value::Error(
                    "ERR arguments must contain a value for every field".to_string(),
                ))
            }
        }
    }
    // TODO: check for other value types for the value
    let mut cache = server.cache.lock().unwrap();
    match cache.get_mut(&key) {
        Some(item) => {
            if let RedisType::Hash = item.redis_type {
                let mut count = 0;
                if let Value::Hash(ref mut hash) = item.value {
                    for chunk in args.chunks(2) {
                        match chunk {
                            [Value::BulkString(field), value] => {
                                println!("Field: {:?}, Value: {:?}", field, value);
                                hash.entry(field.to_string())
                                    .or_insert_with(|| value.clone());
                                count += 1;
                                Some(Value::BulkString("Ok".to_string()))
                            }
                            _ => Some(Value::Error(
                                "Arguments must contain a value for every field".to_string(),
                            )),
                        };
                    }
                }
                return Some(Value::Integer(count));
            }
            Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string(),
            ))
        }
        // TODO: handle creation of new hash if no key found
        None => Some(Value::BulkString(
            "TODO: handle creation of new hash if no key found".to_string(),
        )),
    }
    // 2. Lock the cache.
    // 3. Retrieve the hash associated with the key.
    // 4. Set the specified field to the value in the hash.
    // 5. Return 1 if the field is new, 0 if the field existed and was updated.
}
