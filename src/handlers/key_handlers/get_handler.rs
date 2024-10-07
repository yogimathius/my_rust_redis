use crate::{log, models::value::Value, server::Server};
use std::time::Instant;

pub fn get_handler(server: &mut Server, key: String, _args: Vec<Value>) -> Option<Value> {
    let cache = server.cache.lock().unwrap();
    log!("key {:?}", key);
    match cache.get(&key) {
        Some(value) => {
            log!("value {:?}", value);
            let response = if let Some(expiration) = value.expiration {
                let now = Instant::now();
                if now.duration_since(value.created_at).as_millis() > expiration as u128 {
                    Value::NullBulkString
                } else {
                    value.value.clone()
                }
            } else {
                value.value.clone()
            };
            log!("response {:?}", response);
            Some(response)
        }
        None => Some(Value::NullBulkString),
    }
}
