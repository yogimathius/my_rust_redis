use crate::{log, models::value::Value, server::Server};
use std::{sync::Arc, thread};

pub fn unlink_handler(server: &mut Server, _key: String, args: Vec<Value>) -> Option<Value> {
    let keys: Vec<String> = args
        .into_iter()
        .filter_map(|arg| match arg {
            Value::BulkString(s) => Some(s),
            _ => None,
        })
        .collect();
    let cache = Arc::clone(&server.cache);
    thread::spawn(move || {
        let mut cache = cache.lock().unwrap();
        log!("keys {:?}", keys);

        for key in keys {
            cache.remove(&key);
        }
    });
    Some(Value::SimpleString("OK".to_string()))
}
