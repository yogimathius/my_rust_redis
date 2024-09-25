#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    use my_redis_server::handlers::lindex_handler;
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
    fn test_lindex_handler_get_first() {
        let mut server = setup();
        let args = vec![Value::Integer(0)];
        let result = lindex_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::BulkString("value1".to_string())));
    }

    #[test]
    fn test_lindex_handler_get_last() {
        let mut server = setup();
        let args = vec![Value::Integer(-1)];
        let result = lindex_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::BulkString("value3".to_string())));
    }

    #[test]
    fn test_lindex_handler_out_of_range() {
        let mut server = setup();
        let args = vec![Value::Integer(4)];
        let result = lindex_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::NullBulkString));
    }

    #[test]
    fn test_lindex_handler_negative_one_gets_last() {
        let mut server = setup();
        let args = vec![Value::Integer(-1)];
        let result = lindex_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::BulkString("value3".to_string())));
    }

    #[test]
    fn test_lindex_handler_negative_two_gets_second_to_last() {
        let mut server = setup();
        let args = vec![Value::Integer(-2)];
        let result = lindex_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::BulkString("value2".to_string())));
    }

    #[test]
    fn test_lindex_handler_negative_out_of_range() {
        let mut server = setup();
        let args = vec![Value::Integer(-5)];
        let result = lindex_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::NullBulkString));
    }
}
