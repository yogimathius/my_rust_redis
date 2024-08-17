use crate::{
    model::Value,
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

pub fn keys_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn type_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn del_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn unlink_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn expire_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn rename_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}
