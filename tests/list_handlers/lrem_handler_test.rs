#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    use my_redis_server::handlers::{lrem_handler, lset_handler};
    use my_redis_server::models::redis_type::RedisType;
    use my_redis_server::models::value::Value;
    use my_redis_server::server::{RedisItem, Role, Server};

    fn setup() -> Server {
        let server = Server {
            cache: Arc::new(Mutex::new(HashMap::new())),
            role: Role::Main,
            port: 6379,
            sync: false,
        };
        let key = "key".to_string();

        let list = vec![
            Value::BulkString("value1".to_string()),
            Value::BulkString("value2".to_string()),
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

        server
    }

    #[test]
    fn test_lrem_handler_remove_existing_value() {
        let mut server = setup();
        let key = "key".to_string();

        let lrem_args = vec![Value::Integer(1), Value::BulkString("value1".to_string())];
        let result = lrem_handler(&mut server, key.clone(), lrem_args);
        assert_eq!(result, Some(Value::Integer(1)));
    }

    #[test]
    fn test_lrem_handler_remove_non_existing_value() {
        let mut server = setup();
        let key = "key".to_string();
        let args = vec![
            Value::BulkString(key.clone()),
            Value::BulkString("value".to_string()),
        ];
        lset_handler(&mut server, key.clone(), args);

        let lrem_args = vec![
            Value::Integer(1),
            Value::BulkString("extra_non_existing".to_string()),
        ];
        let result = lrem_handler(&mut server, key.clone(), lrem_args);
        assert_eq!(result, Some(Value::Integer(0)));
    }

    #[test]
    fn test_lrem_handler_remove_multiple_values() {
        let mut server = setup();
        let key = "key".to_string();

        let lrem_args = vec![Value::Integer(2), Value::BulkString("value2".to_string())];
        let result = lrem_handler(&mut server, key.clone(), lrem_args);
        assert_eq!(result, Some(Value::Integer(2)));
    }

    #[test]
    fn test_lrem_handler_invalid_count() {
        let mut server = setup();
        let key = "key".to_string();
        let args = vec![
            Value::BulkString(key.clone()),
            Value::BulkString("value".to_string()),
        ];
        lset_handler(&mut server, key.clone(), args);

        let lrem_args = vec![
            Value::BulkString("invalid_count".to_string()),
            Value::BulkString("value".to_string()),
        ];
        let result = lrem_handler(&mut server, key.clone(), lrem_args);
        assert_eq!(
            result,
            Some(Value::Error("ERR value is not an integer".to_string()))
        );
    }

    #[test]
    fn test_lrem_handler_invalid_value_type() {
        let mut server = setup();
        let key = "key".to_string();
        let args = vec![
            Value::BulkString(key.clone()),
            Value::BulkString("value".to_string()),
        ];
        lset_handler(&mut server, key.clone(), args);

        let lrem_args = vec![Value::Integer(1), Value::Integer(123)];
        let result = lrem_handler(&mut server, key.clone(), lrem_args);
        assert_eq!(
            result,
            Some(Value::Error("ERR value is not a bulk string".to_string()))
        );
    }
}
