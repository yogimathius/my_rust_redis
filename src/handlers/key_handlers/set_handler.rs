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
