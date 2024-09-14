use crate::{handlers::utilities::should_set_expiry, models::value::Value, server::Server};
use std::time::Instant;

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
