use crate::{models::value::Value, server::Server};

pub fn hget_handler(_server: &mut Server, _args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key and field from args.
    // 2. Lock the cache.
    // 3. Retrieve the hash associated with the key.
    // 4. Return the value associated with the field as a BulkString.
    Some(Value::SimpleString("OK".to_string()))
}
