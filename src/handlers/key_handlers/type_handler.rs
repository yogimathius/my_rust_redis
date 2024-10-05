use crate::models::{redis_item::RedisItem, redis_type::RedisType, value::Value};
use std::collections::HashMap;

pub async fn type_handler(
    cache: HashMap<String, RedisItem>,
    key: String,
    _args: Vec<Value>,
) -> Option<Value> {
    if let Some(item) = cache.get(&key) {
        return Some(Value::SimpleString(item.redis_type.to_string()));
    } else {
        return Some(Value::SimpleString(RedisType::None.to_string()));
    }
}
