use crate::{models::value::Value, server::Server};

pub fn llen_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key from args.
    // 2. Lock the cache.
    // 3. Retrieve the list associated with the key.
    // 4. Return the length of the list as an Integer.
    Some(Value::SimpleString("OK".to_string()))
}

pub fn lrem_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key, count, and value to remove from args.
    // 2. Lock the cache.
    // 3. Retrieve the list associated with the key.
    // 4. Remove the specified number of occurrences of the value from the list.
    // 5. Return the number of removed elements as an Integer.
    Some(Value::SimpleString("OK".to_string()))
}

pub fn lindex_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key and index from args.
    // 2. Lock the cache.
    // 3. Retrieve the list associated with the key.
    // 4. Return the element at the specified index as a BulkString.
    Some(Value::SimpleString("OK".to_string()))
}

pub fn lpop_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key from args.
    // 2. Lock the cache.
    // 3. Retrieve the list associated with the key.
    // 4. Remove and return the first element of the list as a BulkString.
    Some(Value::SimpleString("OK".to_string()))
}

pub fn rpop_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key from args.
    // 2. Lock the cache.
    // 3. Retrieve the list associated with the key.
    // 4. Remove and return the last element of the list as a BulkString.
    Some(Value::SimpleString("OK".to_string()))
}

// pub fn lpush_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
//     // Pseudocode:
//     // 1. Extract the key and values to push from args.
//     // 2. Lock the cache.
//     // 3. Retrieve the list associated with the key.
//     // 4. Push the values to the front of the list.
//     // 5. Return the length of the list after the push as an Integer.
//     Some(Value::SimpleString("OK".to_string()))
// }

// pub fn rpush_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
//     // Pseudocode:
//     // 1. Extract the key and values to push from args.
//     // 2. Lock the cache.
//     // 3. Retrieve the list associated with the key.
//     // 4. Push the values to the end of the list.
//     // 5. Return the length of the list after the push as an Integer.
//     Some(Value::SimpleString("OK".to_string()))
// }

pub fn lset_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    // Pseudocode:
    // 1. Extract the key, index, and value from args.
    // 2. Lock the cache.
    // 3. Retrieve the list associated with the key.
    // 4. Set the element at the specified index to the new value.
    // 5. Return OK if successful.
    Some(Value::SimpleString("OK".to_string()))
}
