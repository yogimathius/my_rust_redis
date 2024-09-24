#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use my_redis_server::handlers::{hget_handler, hset_handler};
    use my_redis_server::models::value::Value;
    use my_redis_server::server::{Role, Server};

    fn setup() -> Server {
        Server {
            cache: Arc::new(Mutex::new(HashMap::new())),
            role: Role::Main,
            port: 6379,
            sync: false,
        }
    }

    #[test]
    fn test_hget_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("field".to_string()),
            Value::BulkString("value".to_string()),
        ];
        hset_handler(&mut server, "key".to_string(), args);
        let args = vec![Value::BulkString("field".to_string())];
        let result = hget_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::BulkString("value".to_string())));
    }

    #[test]
    fn test_hget_non_existent_key() {
        let mut server = setup();
        let args = vec![Value::BulkString("field".to_string())];
        let result = hget_handler(&mut server, "non_existent_key".to_string(), args);
        assert_eq!(result, None);
    }

    #[test]
    fn test_hget_non_hash_type_key() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("field".to_string()),
            Value::BulkString("value".to_string()),
        ];
        hset_handler(&mut server, "key".to_string(), args);

        // Simulate setting the key to a different type
        {
            let mut cache = server.cache.lock().unwrap();
            cache.insert(
                "key".to_string(),
                my_redis_server::server::RedisItem {
                    value: Value::BulkString("some string".to_string()),
                    created_at: std::time::Instant::now(),
                    expiration: None,
                    redis_type: my_redis_server::models::redis_type::RedisType::String,
                },
            );
        }

        let args = vec![Value::BulkString("field".to_string())];
        let result = hget_handler(&mut server, "key".to_string(), args);
        assert_eq!(
            result,
            Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string()
            ))
        );
    }

    #[test]
    fn test_hget_non_existent_field() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("field".to_string()),
            Value::BulkString("value".to_string()),
        ];
        hset_handler(&mut server, "key".to_string(), args);
        let args = vec![Value::BulkString("non_existent_field".to_string())];
        let result = hget_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, None);
    }

    #[test]
    fn test_hget_invalid_arguments() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("field".to_string()),
            Value::BulkString("value".to_string()),
        ];
        hset_handler(&mut server, "key".to_string(), args);
        let args = vec![Value::Integer(10)];
        let result = hget_handler(&mut server, "key".to_string(), args);
        assert_eq!(
            result,
            Some(Value::Error(
                "ERR arguments must contain a value for every field".to_string()
            ))
        );
    }
}
