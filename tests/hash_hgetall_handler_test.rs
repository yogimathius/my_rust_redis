#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use my_redis_server::handlers::{hgetall_handler, hset_handler};
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
    fn test_hgetall_multiple_fields() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("field1".to_string()),
            Value::BulkString("value1".to_string()),
            Value::BulkString("field2".to_string()),
            Value::BulkString("value2".to_string()),
        ];
        hset_handler(&mut server, "key".to_string(), args);
        let result = hgetall_handler(&mut server, "key".to_string(), vec![]);
        assert_eq!(
            result,
            Some(Value::Array(vec![
                Value::BulkString("field1".to_string()),
                Value::BulkString("value1".to_string()),
                Value::BulkString("field2".to_string()),
                Value::BulkString("value2".to_string())
            ]))
        );
    }

    #[test]
    fn test_hgetall_empty_hash() {
        let mut server = setup();
        let args = vec![];
        hset_handler(&mut server, "key".to_string(), args);
        let result = hgetall_handler(&mut server, "key".to_string(), vec![]);
        assert_eq!(result, Some(Value::Array(vec![])));
    }

    #[test]
    fn test_hgetall_non_existent_key() {
        let mut server = setup();
        let result = hgetall_handler(&mut server, "non_existent_key".to_string(), vec![]);
        assert_eq!(result, Some(Value::Array(vec![])));
    }

    #[test]
    fn test_hgetall_non_hash_type_key() {
        let mut server = setup();
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
        let result = hgetall_handler(&mut server, "key".to_string(), vec![]);
        assert_eq!(
            result,
            Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string()
            ))
        );
    }
}
