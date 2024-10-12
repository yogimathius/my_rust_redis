use super::hash_utils::HashOperation;
use crate::{models::value::Value, server::Server};

pub fn hgetall_handler(server: &mut Server, key: String, _: Vec<Value>) -> Option<Value> {
    server.operate_on_hash(&key, |hash| {
        let mut sorted_keys: Vec<_> = hash.keys().cloned().collect();
        sorted_keys.sort();

        let hash_arr: Vec<Value> = sorted_keys
            .into_iter()
            .flat_map(|k| {
                let v = hash.get(&k).unwrap();
                vec![Value::BulkString(k), v.clone()]
            })
            .collect();

        Some(Value::Array(hash_arr))
    })
}
