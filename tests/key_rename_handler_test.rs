#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    use my_redis_server::handlers::rename_handler;
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

        let fixed_instant = Instant::now() - Duration::from_secs(1000);

        server.cache.lock().unwrap().insert(
            "old_key".to_string(),
            RedisItem {
                value: Value::BulkString("some string".to_string()),
                created_at: fixed_instant,
                expiration: None,
                redis_type: RedisType::String,
            },
        );

        server
    }

    fn bulk_string(value: &str) -> Value {
        Value::BulkString(value.to_string())
    }

    #[test]
    fn test_rename_success() {
        let mut server = setup();

        let args = vec![bulk_string("old_key"), bulk_string("new_key")];
        let result = rename_handler(&mut server, "old_key".to_string(), args);
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));

        let cache = server.cache.lock().unwrap();
        assert!(cache.contains_key("new_key"));
        assert!(!cache.contains_key("old_key"));
        assert_eq!(
            cache.get("new_key").map(|item| &item.value),
            Some(&Value::BulkString("some string".to_string()))
        );
    }

    #[test]
    fn test_rename_key_does_not_exist() {
        let mut server = setup();
        let args = vec![bulk_string("non_existent_key"), bulk_string("new_key")];
        let result = rename_handler(&mut server, "non_existent_key".to_string(), args);
        assert_eq!(result, Some(Value::Error("ERR no such key".to_string())));
    }

    #[test]
    fn test_rename_new_key_already_exists() {
        let mut server = setup();
        let fixed_instant = Instant::now() - Duration::from_secs(1000);
        server.cache.lock().unwrap().insert(
            "new_key".to_string(),
            RedisItem {
                value: Value::BulkString("existing_value".to_string()),
                created_at: fixed_instant,
                expiration: None,
                redis_type: RedisType::String,
            },
        );
        let args = vec![bulk_string("old_key"), bulk_string("new_key")];
        let result = rename_handler(&mut server, "old_key".to_string(), args);
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));

        let cache = server.cache.lock().unwrap();
        assert!(cache.contains_key("new_key"));
        assert!(!cache.contains_key("old_key"));
        assert_eq!(
            cache.get("new_key").map(|item| &item.value),
            Some(&Value::BulkString("some string".to_string()))
        );
    }
}
