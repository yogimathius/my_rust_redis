use super::hash_utils::HashOperation;
use crate::{models::value::Value, server::Server};

pub fn hvals_handler(server: &mut Server, key: String, _: Vec<Value>) -> Option<Value> {
    server.operate_on_hash(&key, |hash| {
        let mut values: Vec<_> = hash.values().cloned().collect();
        values.sort_by(|a, b| a.clone().serialize().cmp(&b.clone().serialize())); // Custom comparison
        Some(Value::Array(values))
    })
}
