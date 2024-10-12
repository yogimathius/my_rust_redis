use super::hash_utils::HashOperation;
use crate::{models::value::Value, server::Server};

pub fn hkeys_handler(server: &mut Server, key: String, _: Vec<Value>) -> Option<Value> {
    server.operate_on_hash(&key, |hash| {
        let mut keys: Vec<_> = hash.keys().cloned().collect();
        keys.sort();
        let keys = keys.into_iter().map(Value::BulkString).collect();
        Some(Value::Array(keys))
    })
}
