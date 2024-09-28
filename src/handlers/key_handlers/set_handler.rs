use crate::{
    log,
    models::{redis_type::RedisType, value::Value},
    server::{RedisItem, Server},
    utilities::unpack_bulk_str,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use std::time::Instant;

pub async fn set_handler(
    server: Arc<Mutex<Server>>,
    key: String,
    args: Vec<Value>,
) -> Option<Value> {
    let server = server.lock().await;

    println!("args {:?}", args);
    let value = Value::BulkString(unpack_bulk_str(args.get(0).unwrap().clone()).unwrap());
    let option = match args.get(1) {
        Some(value) => unpack_bulk_str(value.clone()),
        None => unpack_bulk_str(Value::BulkString("".to_string())),
    };
    let mut cache = server.cache.lock().await;

    let expiration: Option<i64> = match option.unwrap().as_str() {
        "EX" | "px" => {
            let expiration_str = unpack_bulk_str(args.get(2).unwrap().clone()).unwrap();
            println!("expiration_str {:?}", expiration_str);
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
    println!("key {:?}", key);
    println!("value {:?}", redis_item);
    cache.insert(key, redis_item);
    log!("Ok");
    Some(Value::SimpleString("OK".to_string()))
}
