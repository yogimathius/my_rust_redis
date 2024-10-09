use crate::{log, models::value::Value, server::Server};
use std::time::Instant;

pub fn get_handler(server: &mut Server, key: String, _args: Vec<Value>) -> Option<Value> {
    log!("key {:?}", key);
    let cache = server.cache.lock().unwrap();
    match cache.get(&key) {
        Some(item) => {
            log!("value {:?}", item);
            if let Some(expiration) = item.expiration {
                if Instant::now().duration_since(item.created_at).as_secs() as i64 >= expiration {
                    return Some(Value::NullBulkString);
                }
            }
            log!("response {:?}", item.value);
            Some(item.value.clone())
        }
        None => Some(Value::NullBulkString),
    }
}
