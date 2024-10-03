use crate::log;
use crate::models::value::Value;
use crate::server::RedisItem;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

pub async fn get_handler(
    cache: Arc<Mutex<HashMap<String, RedisItem>>>,
    key: String,
    _args: Vec<Value>,
) -> Option<Value> {
    let cache = cache.lock().await;
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
