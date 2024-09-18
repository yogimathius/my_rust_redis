use crate::{
    log,
    models::{redis_type::RedisType, value::Value},
    server::{RedisItem, Server},
    utilities::{unpack_bulk_str, unpack_integer},
};
use std::time::Instant;

pub fn set_handler(server: &mut Server, _key: String, args: Vec<Value>) -> Option<Value> {
    let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
    let value = Value::BulkString(unpack_bulk_str(args.get(1).unwrap().clone()).unwrap());
    let option = match args.get(2) {
        Some(value) => unpack_bulk_str(value.clone()),
        None => unpack_bulk_str(Value::BulkString("".to_string())),
    };
    let mut cache = server.cache.lock().unwrap();

    let expiration_time: Option<i64> = match option.unwrap().as_str() {
        "EX" => Some(unpack_integer(args.get(3).unwrap().clone()).unwrap()),
        "PX" => Some(unpack_integer(args.get(3).unwrap().clone()).unwrap()),
        _ => None,
    };
    log!("expiration_time {:?}", expiration_time);
    let redis_item = RedisItem {
        value,
        created_at: Instant::now(),
        expiration: expiration_time,
        redis_type: RedisType::String,
    };

    cache.insert(key, redis_item);
    log!("Ok");
    Some(Value::SimpleString("OK".to_string()))
}
