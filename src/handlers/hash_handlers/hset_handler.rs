use std::{collections::HashMap, time::Instant};

use crate::{
    log,
    models::{redis_item::RedisItem, redis_type::RedisType, value::Value},
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
    let mut cache = server.cache.lock().unwrap();
    match cache.get_mut(&key) {
        Some(item) => {
            if let RedisType::Hash = item.redis_type {
                let mut count = 0;
                if let Value::Hash(ref mut hash) = item.value {
                    for chunk in args.chunks(2) {
                        match chunk {
                            [Value::BulkString(field), value] => {
                                log!("Field: {:?}, Value: {:?}", field, value);
                                let entry = hash.entry(field.to_string());
                                log!("Entry: {:?}", entry);
                                let inserted_value = hash.insert(field.to_string(), value.clone());
                                log!("Entry after insert: {:?}", inserted_value);

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
        None => {
            let mut hash = HashMap::new();
            let mut count = 0;
            for chunk in args.chunks(2) {
                if let [Value::BulkString(field), value] = chunk {
                    hash.insert(field.to_string(), value.clone());
                    count += 1;
                }
            }
            let redis_item = RedisItem::new_hash(hash);
            cache.insert(key, redis_item);
            Some(Value::Integer(count))
        }
    }
}
