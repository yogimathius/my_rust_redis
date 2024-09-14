#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use my_redis_server::handlers::{rename_handler, set_handler};
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
    fn test_rename_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        set_handler(&mut server, args);
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("new_key".to_string()),
        ];
        let result = rename_handler(&mut server, args);
        // This is a placeholder assertion, update it with the actual expected result
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));
    }
}
