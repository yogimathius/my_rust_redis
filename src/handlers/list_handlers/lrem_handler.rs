use crate::{models::value::Value, server::Server};

pub fn lrem_handler(_server: &mut Server, _key: String, _args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key, count, and value to remove from args.
    // 2. Lock the cache.
    // 3. Retrieve the list associated with the key.
    // 4. Remove the specified number of occurrences of the value from the list.
    // 5. Return the number of removed elements as an Integer.
    Some(Value::SimpleString("OK".to_string()))
}
