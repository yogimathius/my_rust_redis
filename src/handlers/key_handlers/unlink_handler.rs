use std::collections::HashMap;

use crate::models::{redis_item::RedisItem, value::Value};
pub async fn unlink_handler(
    mut cache: HashMap<String, RedisItem>,
    _key: String,
    args: Vec<Value>,
) -> Option<Value> {
    let keys: Vec<String> = args
        .into_iter()
        .filter_map(|arg| match arg {
            Value::BulkString(s) => Some(s),
            _ => None,
        })
        .collect();
    tokio::spawn(async move {
        for key in keys {
            cache.remove(&key);
        }
    });
    Some(Value::SimpleString("OK".to_string()))
}
