use crate::{
    models::{redis_type::RedisType, value::Value},
    server::Server,
    utilities::lock_and_get_item,
};
use std::sync::Arc;
use tokio::sync::Mutex;

// TODO: handle creating a new key if key isn't found
pub async fn lset_handler(
    server: Arc<Mutex<Server>>,
    key: String,
    args: Vec<Value>,
) -> Option<Value> {
    let server = server.lock().await;

    let index = match args.get(1) {
        Some(Value::Integer(i)) => *i as usize,
        _ => return Some(Value::Error("ERR index is not an integer".to_string())),
    };

    let new_value = match args.get(2) {
        Some(v) => v.clone(),
        _ => {
            return Some(Value::Error(
                "ERR wrong number of arguments for 'lset' command".to_string(),
            ))
        }
    };

    match lock_and_get_item(&server.cache, &key, |item| {
        if let RedisType::List = item.redis_type {
            if let Value::Array(ref mut list) = item.value {
                if index < list.len() {
                    list[index] = new_value;
                    return Some(Value::SimpleString("OK".to_string()));
                } else {
                    return Some(Value::Error("ERR index out of range".to_string()));
                }
            }
        }
        Some(Value::Error(
            "ERR operation against a key holding the wrong kind of value".to_string(),
        ))
    })
    .await
    {
        Ok(result) => result,
        Err(err) => Some(err),
    }
}
