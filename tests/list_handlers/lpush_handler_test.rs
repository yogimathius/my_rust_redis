#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    use redis_starter_rust::handlers::lpush_handler;
    use redis_starter_rust::models::redis_type::RedisType;
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::{RedisItem, Role, Server};

    fn setup() -> Server {
        Server {
            cache: Arc::new(Mutex::new(HashMap::new())),
            role: Role::Main,
            port: 6379,
            sync: false,
        }
    }

    #[test]
    fn test_lpush_handler_existing_list() {
        let mut server = setup();
        let key = "key".to_string();
        let initial_list = vec![Value::BulkString("initial".to_string())];
        let redis_item = RedisItem {
            value: Value::Array(initial_list),
            expiration: None,
            created_at: Instant::now(),
            redis_type: RedisType::List,
        };

        server.cache.lock().unwrap().insert(key.clone(), redis_item);

        let args = vec![Value::BulkString("new_item".to_string())];
        let result = lpush_handler(&mut server, key.clone(), args);
        assert_eq!(result, Some(Value::Integer(2)));

        let cache = server.cache.lock().unwrap();
        if let Some(item) = cache.get(&key) {
            if let Value::Array(list) = &item.value {
                assert_eq!(list.len(), 2);
                assert_eq!(list[0], Value::BulkString("new_item".to_string()));
                assert_eq!(list[1], Value::BulkString("initial".to_string()));
            } else {
                panic!("Value is not an array");
            }
        } else {
            panic!("Key not found in cache");
        }
    }

    #[test]
    fn test_lpush_handler_new_list() {
        let mut server = setup();
        let key = "key".to_string();
        let args = vec![Value::BulkString("new_item".to_string())];
        let result = lpush_handler(&mut server, key.clone(), args);
        assert_eq!(result, Some(Value::Integer(1)));

        let cache = server.cache.lock().unwrap();
        if let Some(item) = cache.get(&key) {
            if let Value::Array(list) = &item.value {
                assert_eq!(list.len(), 1);
                assert_eq!(list[0], Value::BulkString("new_item".to_string()));
            } else {
                panic!("Value is not an array");
            }
        } else {
            panic!("Key not found in cache");
        }
    }

    #[test]
    fn test_lpush_handler_invalid_value_type() {
        let mut server = setup();
        let key = "key".to_string();
        let args = vec![Value::Integer(123)];
        let result = lpush_handler(&mut server, key.clone(), args);
        assert_eq!(
            result,
            Some(Value::Error("ERR value is not a bulk string".to_string()))
        );
    }

    #[test]
    fn test_lpush_handler_non_list_value() {
        let mut server = setup();
        let key = "key".to_string();
        let redis_item = RedisItem {
            value: Value::Integer(123),
            expiration: None,
            created_at: Instant::now(),
            redis_type: RedisType::String,
        };

        server.cache.lock().unwrap().insert(key.clone(), redis_item);

        let args = vec![Value::BulkString("new_item".to_string())];
        let result = lpush_handler(&mut server, key.clone(), args);
        assert_eq!(
            result,
            Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string()
            ))
        );
    }
}
