#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use redis_starter_rust::handlers::lset_handler;
    use redis_starter_rust::models::redis_type::RedisType;
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::{RedisItem, Role, Server};
    use std::time::Instant;

    fn setup() -> Server {
        Server {
            cache: Arc::new(Mutex::new(HashMap::new())),
            role: Role::Main,
            port: 6379,
            sync: false,
        }
    }

    #[test]
    fn test_lset_handler() {
        let mut server = setup();

        // Insert a list into the cache
        let key = "key".to_string();
        let list = vec![
            Value::BulkString("value1".to_string()),
            Value::BulkString("value2".to_string()),
            Value::BulkString("value3".to_string()),
        ];
        let redis_item = RedisItem {
            value: Value::Array(list),
            created_at: Instant::now(),
            expiration: None,
            redis_type: RedisType::List,
        };
        server.cache.lock().unwrap().insert(key.clone(), redis_item);

        // Test setting a value in the list
        let args = vec![
            Value::BulkString(key.clone()),
            Value::Integer(1),
            Value::BulkString("new_value".to_string()),
        ];
        let result = lset_handler(&mut server, key.clone(), args);
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));

        // Verify the value was set correctly
        let cache = server.cache.lock().unwrap();
        let item = cache.get(&key).unwrap();
        if let Value::Array(ref list) = item.value {
            assert_eq!(list[1], Value::BulkString("new_value".to_string()));
        } else {
            panic!("Expected list value");
        }
    }

    #[test]
    fn test_lset_handler_index_out_of_range() {
        let mut server = setup();

        // Insert a list into the cache
        let key = "key".to_string();
        let list = vec![
            Value::BulkString("value1".to_string()),
            Value::BulkString("value2".to_string()),
            Value::BulkString("value3".to_string()),
        ];
        let redis_item = RedisItem {
            value: Value::Array(list),
            created_at: Instant::now(),
            expiration: None,
            redis_type: RedisType::List,
        };
        server.cache.lock().unwrap().insert(key.clone(), redis_item);

        // Test setting a value with an out-of-range index
        let args = vec![
            Value::BulkString(key.clone()),
            Value::Integer(10),
            Value::BulkString("new_value".to_string()),
        ];
        let result = lset_handler(&mut server, key, args);
        assert_eq!(
            result,
            Some(Value::Error("ERR index out of range".to_string()))
        );
    }

    #[test]
    fn test_lset_handler_no_such_key() {
        let mut server = setup();

        // Test setting a value in a non-existent list
        let args = vec![
            Value::BulkString("non_existent_key".to_string()),
            Value::Integer(1),
            Value::BulkString("new_value".to_string()),
        ];
        let result = lset_handler(&mut server, "non_existent_key".to_string(), args);
        assert_eq!(result, Some(Value::Error("ERR no such key".to_string())));
    }

    #[test]
    fn test_lset_handler_wrong_type() {
        let mut server = setup();

        // Insert a non-list value into the cache
        let key = "key".to_string();
        let redis_item = RedisItem {
            value: Value::BulkString("value".to_string()),
            created_at: Instant::now(),
            expiration: None,
            redis_type: RedisType::String,
        };
        server.cache.lock().unwrap().insert(key.clone(), redis_item);

        // Test setting a value in a non-list key
        let args = vec![
            Value::BulkString(key.clone()),
            Value::Integer(1),
            Value::BulkString("new_value".to_string()),
        ];
        let result = lset_handler(&mut server, key, args);
        assert_eq!(
            result,
            Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string()
            ))
        );
    }
}
