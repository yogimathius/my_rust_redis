use crate::{models::value::Value, server::Server};

pub fn hgetall_handler(_server: &mut Server, _args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key from args.
    // 2. Lock the cache.
    // 3. Retrieve the hash associated with the key.
    // 4. Return all fields and values in the hash as a BulkString array.
    Some(Value::SimpleString("OK".to_string()))
}
