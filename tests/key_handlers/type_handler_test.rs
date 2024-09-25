#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use redis_starter_rust::handlers::{set_handler, type_handler};
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
    fn test_type_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        set_handler(&mut server, "key".to_string(),  args);
        let args = vec![Value::BulkString("key".to_string())];
        let result = type_handler(&mut server, "key".to_string(),  args);
        assert_eq!(result, Some(Value::SimpleString("string".to_string())));
    }
}
