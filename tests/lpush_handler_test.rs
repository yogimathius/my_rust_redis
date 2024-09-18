#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use my_redis_server::handlers::{lpush_handler, lset_handler};
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
    fn test_lpush_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        lset_handler(&mut server, "key".to_string(),  args);
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("10".to_string()),
        ];
        let result = lpush_handler(&mut server, "key".to_string(),  args);
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));
    }
}
