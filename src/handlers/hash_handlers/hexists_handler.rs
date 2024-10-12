use super::hash_utils::HashOperation;
use crate::{models::value::Value, server::Server};

pub fn hexists_handler(server: &mut Server, key: String, args: Vec<Value>) -> Option<Value> {
    match args.get(0) {
        Some(Value::BulkString(field)) => {
            Some(
                server
                    .operate_on_hash(&key, |hash| {
                        Some(Value::Integer(hash.contains_key(field) as i64))
                    })
                    .unwrap_or(Value::Integer(0)),
            ) // Handle non-existent keys
        }
        _ => Some(Value::Error(
            "ERR arguments must contain a value for every field".to_string(),
        )),
    }
}
