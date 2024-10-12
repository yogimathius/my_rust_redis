use crate::{log, models::value::Value, server::Server};

pub fn unlink_handler(server: &mut Server, _: String, args: Vec<Value>) -> Option<Value> {
    let keys: Vec<String> = args
        .into_iter()
        .filter_map(|arg| match arg {
            Value::BulkString(s) => Some(s),
            _ => None,
        })
        .collect();

    log!("keys {:?}", keys);

    let mut removed_count = 0;
    {
        let mut cache = server.cache.lock().unwrap();
        for key in keys {
            if cache.remove(&key).is_some() {
                log!("removed key {}", key);
                removed_count += 1;
            } else {
                log!("key {} not found", key);
            }
        }
    }

    Some(Value::Integer(removed_count))
}
