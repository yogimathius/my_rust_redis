use chrono::Utc;

use crate::{log, models::value::Value, server::Server};
use std::time::Instant;

pub fn get_handler(server: &mut Server, key: String, _args: Vec<Value>) -> Option<Value> {
    log!("key {:?}", key);
    let cache = server.cache.lock().unwrap();
    match cache.get(&key) {
        Some(item) => {
            log!("value {:?}", item);
            if let Some(expiration) = item.expiration {
                let current_time = Utc::now().timestamp();
                if current_time >= item.created_at + expiration {
                    return Some(Value::NullBulkString);
                }
            }
            log!("response {:?}", item.value);
            Some(item.value.clone())
        }
        None => Some(Value::NullBulkString),
    }
}
