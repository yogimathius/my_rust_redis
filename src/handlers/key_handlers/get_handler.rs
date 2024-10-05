use crate::log;
use crate::models::redis_item::RedisItem;
use crate::models::value::Value;
use std::collections::HashMap;

use std::time::Instant;

pub async fn get_handler(
    cache: HashMap<String, RedisItem>,
    key: String,
    _args: Vec<Value>,
) -> Option<Value> {
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
