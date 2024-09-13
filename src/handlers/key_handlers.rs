use crate::{
    models::{redis_type::RedisType, value::Value},
    server::{RedisItem, Server},
    utilities::{get_expiration, unpack_bulk_str},
};
use std::time::Instant;

pub fn set_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
    let value = unpack_bulk_str(args.get(1).unwrap().clone()).unwrap();
    let mut cache = server.cache.lock().unwrap();

    let expiration_time: Option<i64> = get_expiration(args).unwrap_or(None);

    let redis_item = RedisItem {
        value,
        created_at: Instant::now(),
        expiration: expiration_time,
        redis_type: RedisType::String,
    };

    cache.insert(key, redis_item);
    println!("Ok");
    Some(Value::SimpleString("OK".to_string()))
}

pub fn get_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
    let cache = server.cache.lock().unwrap();
    match cache.get(&key) {
        Some(value) => {
            let response = if let Some(expiration) = value.expiration {
                let now = Instant::now();
                if now.duration_since(value.created_at).as_millis() > expiration as u128 {
                    Value::NullBulkString
                } else {
                    Value::BulkString(value.value.clone())
                }
            } else {
                Value::BulkString(value.value.clone())
            };
            Some(response)
        }
        None => Some(Value::NullBulkString),
    }
}

pub fn keys_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    println!("keys_handler handler {:?}", args);
    // Pseudocode:
    // 1. Extract pattern from args.
    // 2. Lock the cache.
    // 3. Iterate over keys in the cache and match them against the pattern.
    // 4. Collect matching keys.
    // 5. Return matching keys as a BulkString array.
    Some(Value::Array(vec![]))
}

pub fn type_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    if let Some(Value::BulkString(key)) = args.get(0) {
        let cache = server.cache.lock().unwrap();
        if let Some(item) = cache.get(key) {
            return Some(Value::SimpleString(item.redis_type.to_string()));
        } else {
            return Some(Value::SimpleString(RedisType::None.to_string()));
        }
    }

    Some(Value::SimpleString(RedisType::None.to_string()))
}

pub fn del_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    println!("del_handler handler {:?}", args);

    // Pseudocode:
    // 1. Extract key from args.
    // 2. Lock the cache.
    // 3. Remove the key from the cache.
    // 4. Return the number of keys removed as an Integer.
    Some(Value::SimpleString("OK".to_string()))
}

pub fn unlink_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    println!("unlink_handler handler {:?}", args);

    // Pseudocode:
    // 1. Extract key from args.
    // 2. Lock the cache.
    // 3. Remove the key from the cache asynchronously.
    // 4. Return the number of keys removed as an Integer.
    Some(Value::SimpleString("OK".to_string()))
}

pub fn expire_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    println!("expire_handler handler {:?}", args);

    // Pseudocode:
    // 1. Extract key and expiration time from args.
    // 2. Lock the cache.
    // 3. Set the expiration time for the key.
    // 4. Return 1 if the timeout was set, 0 if the key does not exist.
    Some(Value::SimpleString("OK".to_string()))
}

pub fn rename_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    println!("rename_handler handler {:?}", args);

    // Pseudocode:
    // 1. Extract old key and new key from args.
    // 2. Lock the cache.
    // 3. Rename the key in the cache.
    // 4. Return OK if successful.
    Some(Value::SimpleString("OK".to_string()))
}
