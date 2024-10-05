use std::{collections::HashMap, time::Instant};

use crate::{
    log,
    models::{redis_item::RedisItem, redis_type::RedisType, value::Value},
};

pub async fn hset_handler(
    mut cache: HashMap<String, RedisItem>,
    key: String,
    args: Vec<Value>,
) -> Option<Value> {
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
            let redis_item = RedisItem {
                value: Value::Hash(hash),
                created_at: Instant::now(),
                expiration: None,
                redis_type: RedisType::Hash,
            };
            cache.insert(key, redis_item);
            Some(Value::Integer(count))
        }
    }
    // 2. Lock the cache.
    // 3. Retrieve the hash associated with the key.
    // 4. Set the specified field to the value in the hash.
    // 5. Return 1 if the field is new, 0 if the field existed and was updated.
}
