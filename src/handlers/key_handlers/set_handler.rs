use crate::{
    log,
    models::{redis_type::RedisType, value::Value},
    server::{RedisItem, Server},
    utilities::unpack_integer,
};
use std::time::Instant;

pub fn set_handler(server: &mut Server, key: String, args: Vec<Value>) -> Option<Value> {
    log!("args {:?}", args);
    let value = match args.get(0) {
        Some(Value::BulkString(v)) => v.clone(),
        _ => return Some(Value::Error("ERR invalid value".into())),
    };
    let expiration: Option<i64> = match args.get(3) {
        Some(value) => unpack_integer(value.clone()).ok(),
        None => None,
    };
    log!("expiration {:?}", expiration);
    let mut cache = server.cache.lock().unwrap();

    let item = RedisItem {
        value: Value::BulkString(value),
        created_at: Instant::now().elapsed().as_secs() as i64,
        expiration,
        redis_type: RedisType::String,
    };

    cache.insert(key, item);
    log!("Ok");
    Some(Value::SimpleString("OK".to_string()))
}
