use crate::{models::value::Value, server::Server, utilities::unpack_bulk_str};
use std::time::Instant;

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
                    value.value.clone()
                }
            } else {
                value.value.clone()
            };
            Some(response)
        }
        None => Some(Value::NullBulkString),
    }
}
