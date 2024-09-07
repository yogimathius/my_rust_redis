use crate::{model::Value, server::Server};

pub fn hget_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key and field from args.
    // 2. Lock the cache.
    // 3. Retrieve the hash associated with the key.
    // 4. Return the value associated with the field as a BulkString.
    Some(Value::SimpleString("OK".to_string()))
}

pub fn hexists_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key and field from args.
    // 2. Lock the cache.
    // 3. Retrieve the hash associated with the key.
    // 4. Check if the field exists in the hash.
    // 5. Return 1 if the field exists, 0 otherwise.
    Some(Value::SimpleString("OK".to_string()))
}

pub fn hdel_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key and field(s) from args.
    // 2. Lock the cache.
    // 3. Retrieve the hash associated with the key.
    // 4. Remove the specified field(s) from the hash.
    // 5. Return the number of fields removed as an Integer.
    Some(Value::SimpleString("OK".to_string()))
}

pub fn hgetall_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key from args.
    // 2. Lock the cache.
    // 3. Retrieve the hash associated with the key.
    // 4. Return all fields and values in the hash as a BulkString array.
    Some(Value::SimpleString("OK".to_string()))
}

pub fn hkeys_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key from args.
    // 2. Lock the cache.
    // 3. Retrieve the hash associated with the key.
    // 4. Return all fields in the hash as a BulkString array.
    Some(Value::SimpleString("OK".to_string()))
}

pub fn hlen_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key from args.
    // 2. Lock the cache.
    // 3. Retrieve the hash associated with the key.
    // 4. Return the number of fields in the hash as an Integer.
    Some(Value::SimpleString("OK".to_string()))
}

pub fn hmset_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key and field-value pairs from args.
    // 2. Lock the cache.
    // 3. Retrieve the hash associated with the key.
    // 4. Set the specified field-value pairs in the hash.
    // 5. Return OK if successful.
    Some(Value::SimpleString("OK".to_string()))
}

pub fn hset_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key, field, and value from args.
    // 2. Lock the cache.
    // 3. Retrieve the hash associated with the key.
    // 4. Set the specified field to the value in the hash.
    // 5. Return 1 if the field is new, 0 if the field existed and was updated.
    Some(Value::SimpleString("OK".to_string()))
}

pub fn hvals_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key from args.
    // 2. Lock the cache.
    // 3. Retrieve the hash associated with the key.
    // 4. Return all values in the hash as a BulkString array.
    Some(Value::SimpleString("OK".to_string()))
}
