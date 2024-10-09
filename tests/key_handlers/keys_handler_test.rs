use redis_starter_rust::handlers::keys_handler;
use redis_starter_rust::models::redis_type::RedisType;
use redis_starter_rust::models::value::Value;
use redis_starter_rust::server::{RedisItem, Role, Server};
use redis_starter_rust::utilities::ServerState;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[test]
fn test_keys_handler() {
    let mut server = Server {
        cache: Arc::new(Mutex::new(HashMap::new())),
        role: Role::Main,
        port: 6379,
        sync: false,
        server_state: ServerState::StreamingCommands,
    };

    // Populate the cache with some test data
    let mut cache = server.cache.lock().unwrap();
    cache.insert(
        "key1".to_string(),
        RedisItem {
            value: Value::SimpleString("value1".to_string()),
            created_at: Instant::now().elapsed().as_secs() as i64,
            expiration: None,
            redis_type: RedisType::String,
        },
    );
    cache.insert(
        "key2".to_string(),
        RedisItem {
            value: Value::SimpleString("value2".to_string()),
            created_at: Instant::now().elapsed().as_secs() as i64,
            expiration: None,
            redis_type: RedisType::String,
        },
    );
    cache.insert(
        "anotherkey".to_string(),
        RedisItem {
            value: Value::SimpleString("value3".to_string()),
            created_at: Instant::now().elapsed().as_secs() as i64,
            expiration: None,
            redis_type: RedisType::String,
        },
    );
    drop(cache);

    // Helper function to sort and compare results
    fn assert_sorted_results(result: Option<Value>, expected: Vec<&str>) {
        let mut result_keys = match result {
            Some(Value::Array(keys)) => keys,
            _ => vec![],
        };
        result_keys.sort_by(|a, b| match (a, b) {
            (Value::BulkString(a), Value::BulkString(b)) => a.cmp(b),
            _ => std::cmp::Ordering::Equal,
        });

        let mut expected_keys: Vec<Value> = expected
            .into_iter()
            .map(|s| Value::BulkString(s.to_string()))
            .collect();
        expected_keys.sort_by(|a, b| match (a, b) {
            (Value::BulkString(a), Value::BulkString(b)) => a.cmp(b),
            _ => std::cmp::Ordering::Equal,
        });

        assert_eq!(
            Some(Value::Array(result_keys)),
            Some(Value::Array(expected_keys))
        );
    }

    // Test case 1: Match all keys
    let result = keys_handler(
        &mut server,
        "".to_string(),
        vec![Value::BulkString("*".to_string())],
    );
    assert_sorted_results(result, vec!["key1", "key2", "anotherkey"]);

    // Test case 2: Match keys starting with "key"
    let result = keys_handler(
        &mut server,
        "".to_string(),
        vec![Value::BulkString("key*".to_string())],
    );
    assert_sorted_results(result, vec!["key1", "key2"]);

    // Test case 3: Match keys ending with "key"
    let result = keys_handler(
        &mut server,
        "".to_string(),
        vec![Value::BulkString("*key".to_string())],
    );
    assert_sorted_results(result, vec!["anotherkey"]);

    // Test case 4: No matches
    let result = keys_handler(
        &mut server,
        "".to_string(),
        vec![Value::BulkString("nomatch*".to_string())],
    );
    assert_eq!(result, Some(Value::Array(vec![])));

    // Test case 5: Invalid pattern (error case)
    let result = keys_handler(&mut server, "".to_string(), vec![]);
    assert_eq!(
        result,
        Some(Value::Error(
            "ERR wrong number of arguments for 'keys' command".into()
        ))
    );
}
