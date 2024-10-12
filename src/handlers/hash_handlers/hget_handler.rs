use super::hash_utils::HashOperation;
use crate::{models::value::Value, server::Server};

pub fn hget_handler(server: &mut Server, key: String, args: Vec<Value>) -> Option<Value> {
    match args.get(0) {
        Some(Value::BulkString(field)) => {
            Some(
                server
                    .operate_on_hash(&key, |hash| {
                        hash.get(field).cloned().or(Some(Value::NullBulkString))
                    })
                    .unwrap_or(Value::NullBulkString),
            ) // Handle non-existent keys
        }
        _ => Some(Value::Error(
            "ERR arguments must contain a value for every field".to_string(),
        )),
    }
}
