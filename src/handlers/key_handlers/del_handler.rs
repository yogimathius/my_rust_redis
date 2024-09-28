use crate::{models::value::Value, server::Server, utilities::unpack_bulk_str};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn del_handler(
    server: Arc<Mutex<Server>>,
    _key: String,
    args: Vec<Value>,
) -> Option<Value> {
    let server = server.lock().await;

    let keys = args
        .iter()
        .map(|arg| unpack_bulk_str(arg.clone()).unwrap())
        .collect::<Vec<String>>();

    let mut cache = server.cache.lock().await;

    let mut count = 0;

    for key in keys {
        if cache.remove(&key).is_some() {
            count += 1;
        }
    }

    Some(Value::Integer(count))
}
