use crate::{
    models::{redis_type::RedisType, value::Value},
    server::Server,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn type_handler(
    server: Arc<Mutex<Server>>,
    key: String,
    _args: Vec<Value>,
) -> Option<Value> {
    let server = server.lock().await;

    let cache = server.cache.lock().await;
    if let Some(item) = cache.get(&key) {
        return Some(Value::SimpleString(item.redis_type.to_string()));
    } else {
        return Some(Value::SimpleString(RedisType::None.to_string()));
    }
}
