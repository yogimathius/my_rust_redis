#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use redis_starter_rust::handlers::{get_handler, set_handler};
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::{Role, Server};

    fn setup() -> Server {
        Server {
            cache: Arc::new(Mutex::new(HashMap::new())),
            role: Role::Main,
            port: 6379,
            sync: false,
        }
    }

    #[test]
    fn test_get_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        set_handler(&mut server, "key".to_string(), args);
        let args = vec![Value::BulkString("key".to_string())];
        let result = get_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::BulkString("value".to_string())));
    }
}
