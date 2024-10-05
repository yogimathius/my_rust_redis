use crate::models::{redis_item::RedisItem, redis_type::RedisType, value::Value};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub async fn type_handler(
    cache: Arc<Mutex<HashMap<String, RedisItem>>>,
    key: String,
    _args: Vec<Value>,
) -> Option<Value> {
    let cache = cache.lock().await;
    if let Some(item) = cache.get(&key) {
        return Some(Value::SimpleString(item.redis_type.to_string()));
    } else {
        return Some(Value::SimpleString(RedisType::None.to_string()));
    }
}
