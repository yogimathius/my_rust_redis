#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use my_redis_server::handlers::set_handler;
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
    fn test_set_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        let result = set_handler(&mut server, args);
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));
        let cache = server.cache.lock().unwrap();
        assert!(cache.contains_key("key"));
    }

    #[test]
    fn test_set_handler_with_expiration() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
            Value::BulkString("px".to_string()),
            Value::BulkString("10".to_string()),
        ];
        let result = set_handler(&mut server, args);
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));
        let cache = server.cache.lock().unwrap();
        assert!(cache.contains_key("key"));
    }
}
