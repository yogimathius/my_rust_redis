use crate::{log, models::value::Value, server::Server, utilities::unpack_bulk_str};

// Renames key to newkey. It returns an error when key does not exist. If newkey already exists it is overwritten, when this happens RENAME executes an implicit DEL operation, so if the deleted key contains a very big value it may cause high latency even if RENAME itself is usually a constant-time operation.

pub fn rename_handler(server: &mut Server, key: String, args: Vec<Value>) -> Option<Value> {
    log!("rename_handler handler {:?}", args);
    let new_key = unpack_bulk_str(args.get(0).unwrap().clone()).unwrap();

    let mut cache = server.cache.lock().unwrap();

    if !cache.contains_key(&key) {
        return Some(Value::Error("ERR no such key".to_string()));
    }

    if cache.contains_key(&new_key) {
        cache.remove(&new_key);
    }

    if let Some(item) = cache.remove(&key) {
        cache.insert(new_key, item);
        Some(Value::SimpleString("OK".to_string()))
    } else {
        Some(Value::Error("ERR no such key".to_string()))
    }
}
