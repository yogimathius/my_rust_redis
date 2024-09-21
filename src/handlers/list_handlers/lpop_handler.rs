use crate::{models::value::Value, server::Server};

pub fn lpop_handler(server: &mut Server, key: String, _args: Vec<Value>) -> Option<Value> {
    let mut cache = server.cache.lock().unwrap();
    match cache.get_mut(&key) {
        Some(item) => {
            if let Value::Array(ref mut list) = item.value {
                if list.is_empty() {
                    Some(Value::NullBulkString)
                } else {
                    Some(list.remove(0))
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
