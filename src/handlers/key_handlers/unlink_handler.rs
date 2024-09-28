use crate::{models::value::Value, server::Server};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn unlink_handler(
    server: Arc<Mutex<Server>>,
    _key: String,
    args: Vec<Value>,
) -> Option<Value> {
    let server = server.lock().await;

    let keys: Vec<String> = args
        .into_iter()
        .filter_map(|arg| match arg {
            Value::BulkString(s) => Some(s),
            _ => None,
        })
        .collect();
    let cache = Arc::clone(&server.cache);
    tokio::spawn(async move {
        let mut cache = cache.lock().await;

        for key in keys {
            cache.remove(&key);
        }
    });
    Some(Value::SimpleString("OK".to_string()))
}
