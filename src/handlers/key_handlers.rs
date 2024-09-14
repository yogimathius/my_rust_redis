use crate::{
    handlers::utilities::should_set_expiry,
    models::{redis_type::RedisType, value::Value},
    server::{RedisItem, Server},
    utilities::{get_expiration, unpack_bulk_str},
};
use std::{sync::Arc, thread, time::Instant};

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

pub fn keys_handler(_server: &mut Server, args: Vec<Value>) -> Option<Value> {
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
    let keys = args
        .iter()
        .map(|arg| unpack_bulk_str(arg.clone()).unwrap())
        .collect::<Vec<String>>();

    let mut cache = server.cache.lock().unwrap();

    let mut count = 0;

    for key in keys {
        if cache.remove(&key).is_some() {
            count += 1;
        }
    }

    Some(Value::Integer(count))
}

pub fn unlink_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    let keys: Vec<String> = args
        .into_iter()
        .filter_map(|arg| match arg {
            Value::BulkString(s) => Some(s),
            _ => None,
        })
        .collect();
    let cache = Arc::clone(&server.cache);
    thread::spawn(move || {
        let mut cache = cache.lock().unwrap();

        for key in keys {
            cache.remove(&key);
        }
    });
    Some(Value::SimpleString("OK".to_string()))
}

pub fn expire_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    println!("expire_handler handler {:?}", args);
    let key = match args.get(0) {
        Some(Value::BulkString(s)) => s.clone(),
        _ => {
            return Some(Value::Error(
                "ERR wrong number of arguments for 'expire' command".to_string(),
            ))
        }
    };

    let expiration = match args.get(1) {
        Some(Value::BulkString(s)) => s.clone(),
        _ => {
            return Some(Value::Error(
                "ERR wrong number of arguments for 'expire' command".to_string(),
            ))
        }
    };

    let option = match args.get(2) {
        Some(Value::BulkString(s)) => Some(s.clone()),
        _ => None,
    };

    let mut cache = server.cache.lock().unwrap();
    let expiration_time = expiration.parse::<i64>().unwrap();

    match cache.get(&key) {
        Some(value) => {
            println!("value {:?}", value);
            if should_set_expiry(value, expiration_time, option) {
                println!("setting expiration");
                let now = Instant::now();
                let new_expiration =
                    now + std::time::Duration::from_secs(expiration_time.try_into().unwrap());
                let new_expiration_secs =
                    new_expiration.duration_since(Instant::now()).as_secs() as i64;

                let item = cache.get_mut(&key).unwrap();
                item.expiration = Some(new_expiration_secs);
                return Some(Value::Integer(1));
            } else {
                return Some(Value::Integer(0));
            }
        }
        None => return Some(Value::Integer(0)),
    }
}

pub fn rename_handler(_server: &mut Server, args: Vec<Value>) -> Option<Value> {
    println!("rename_handler handler {:?}", args);

    // Pseudocode:
    // 1. Extract old key and new key from args.
    // 2. Lock the cache.
    // 3. Rename the key in the cache.
    // 4. Return OK if successful.
    Some(Value::SimpleString("OK".to_string()))
}
