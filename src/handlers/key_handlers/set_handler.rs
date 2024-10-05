use crate::{
    log,
    models::{redis_item::RedisItem, redis_type::RedisType, value::Value},
    utilities::unpack_bulk_str,
};
use std::collections::HashMap;

use std::time::Instant;

pub async fn set_handler(
    mut cache: HashMap<String, RedisItem>,
    key: String,
    args: Vec<Value>,
) -> Option<Value> {
    log!("args {:?}", args);
    let value = Value::BulkString(unpack_bulk_str(args.get(0).unwrap().clone()).unwrap());
    let option = match args.get(1) {
        Some(value) => unpack_bulk_str(value.clone()),
        None => unpack_bulk_str(Value::BulkString("".to_string())),
    };

    let expiration: Option<i64> = match option.unwrap().as_str() {
        "EX" | "px" => {
            let expiration_str = unpack_bulk_str(args.get(2).unwrap().clone()).unwrap();
            log!("expiration_str {:?}", expiration_str);
            Some(expiration_str.parse::<i64>().unwrap())
        }
        _ => None,
    };
    log!("expiration {:?}", expiration);
    let redis_item = RedisItem {
        value,
        created_at: Instant::now(),
        expiration,
        redis_type: RedisType::String,
    };
    log!("key {:?}", key);
    log!("value {:?}", redis_item);
    cache.insert(key, redis_item);
    log!("Ok");
    Some(Value::SimpleString("OK".to_string()))
}
