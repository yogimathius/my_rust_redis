use crate::{models::value::Value, server::Server};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn rpop_handler(
    server: Arc<Mutex<Server>>,
    key: String,
    _args: Vec<Value>,
) -> Option<Value> {
    let server = server.lock().await;

    let mut cache = server.cache.lock().await;
    match cache.get_mut(&key) {
        Some(item) => {
            if let Value::Array(ref mut list) = item.value {
                if list.is_empty() {
                    Some(Value::NullBulkString)
                } else {
                    Some(list.remove(list.len() - 1))
                }
            } else {
                Some(Value::Error(
                    "ERR operation against a key holding the wrong kind of value".to_string(),
                ))
            }
        }
        None => Some(Value::NullBulkString),
    }
}
