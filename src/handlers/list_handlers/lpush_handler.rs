use crate::{models::value::Value, server::Server};

pub fn lpush_handler(_server: &mut Server, _args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key and values to push from args.
    // 2. Lock the cache.
    // 3. Retrieve the list associated with the key.
    // 4. Push the values to the front of the list.
    // 5. Return the length of the list after the push as an Integer.
    Some(Value::SimpleString("OK".to_string()))
}
