use crate::{models::value::Value, server::Server};

pub fn rpop_handler(_server: &mut Server, _key: String, _args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key from args.
    // 2. Lock the cache.
    // 3. Retrieve the list associated with the key.
    // 4. Remove and return the last element of the list as a BulkString.
    Some(Value::SimpleString("OK".to_string()))
}
