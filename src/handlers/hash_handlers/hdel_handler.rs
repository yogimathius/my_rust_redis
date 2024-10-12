use super::hash_utils::HashOperation;
use crate::{models::value::Value, server::Server};

pub fn hdel_handler(server: &mut Server, key: String, args: Vec<Value>) -> Option<Value> {
    if args.is_empty() {
        return Some(Value::Integer(0));
    }
    server.operate_on_hash(&key, |hash| {
        let mut count = 0;
        for field in args {
            if let Value::BulkString(field) = field {
                if hash.remove(&field).is_some() {
                    count += 1;
                }
            } else {
                return Some(Value::Error(
                    "ERR arguments must contain a value for every field".to_string(),
                ));
            }
        }
        Some(Value::Integer(count))
    })
}
