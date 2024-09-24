#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use my_redis_server::handlers::rename_handler;
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

    fn bulk_string(value: &str) -> Value {
        Value::BulkString(value.to_string())
    }

    #[test]
    fn test_rename_success() {
        let mut server = setup();
        {
            let mut cache = server.cache.lock().unwrap();
            cache.insert(
                "old_key".to_string(),
                Value::BulkString("value".to_string()),
            );
        }
        let args = vec![bulk_string("old_key"), bulk_string("new_key")];
        let result = rename_handler(&mut server, "old_key".to_string(), args);
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));

        let cache = server.cache.lock().unwrap();
        assert!(cache.contains_key("new_key"));
        assert!(!cache.contains_key("old_key"));
        assert_eq!(cache.get("new_key"), Some(&bulk_string("value")));
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
        {
            let mut cache = server.cache.lock().unwrap();
            cache.insert("old_key".to_string(), bulk_string("value"));
            cache.insert("new_key".to_string(), bulk_string("existing_value"));
        }
        let args = vec![bulk_string("old_key"), bulk_string("new_key")];
        let result = rename_handler(&mut server, "old_key".to_string(), args);
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));

        let cache = server.cache.lock().unwrap();
        assert!(cache.contains_key("new_key"));
        assert!(!cache.contains_key("old_key"));
        assert_eq!(cache.get("new_key"), Some(&bulk_string("value")));
    }
}
