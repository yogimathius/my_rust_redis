use crate::{models::value::Value, server::Server};

pub fn hset_handler(_server: &mut Server, _key: String, _args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key, field, and value from args.
    // 2. Lock the cache.
    // 3. Retrieve the hash associated with the key.
    // 4. Set the specified field to the value in the hash.
    // 5. Return 1 if the field is new, 0 if the field existed and was updated.
    Some(Value::SimpleString("OK".to_string()))
}
