#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use redis_starter_rust::handlers::{del_handler, set_handler};
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
    fn test_del_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key1".to_string()),
            Value::BulkString("value1".to_string()),
        ];
        set_handler(&mut server, "key1".to_string(), args);
        let args = vec![
            Value::BulkString("key2".to_string()),
            Value::BulkString("value2".to_string()),
        ];
        set_handler(&mut server, "key2".to_string(), args);

        let args = vec![
            Value::BulkString("key1".to_string()),
            Value::BulkString("key2".to_string()),
            Value::BulkString("nonexistent_key".to_string()),
        ];
        let result = del_handler(&mut server, "key1".to_string(), args);
        assert_eq!(result, Some(Value::Integer(2)));

        let args = vec![Value::BulkString("nonexistent_key".to_string())];
        let result = del_handler(&mut server, "nonexistent_key".to_string(), args);
        assert_eq!(result, Some(Value::Integer(0)));
    }
}
