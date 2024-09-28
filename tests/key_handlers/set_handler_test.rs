#[cfg(test)]
mod tests {
 

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::set_handler;
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;

    fn setup() -> Server {
        return setup_server();
    }

    #[test]
    fn test_set_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        let result = set_handler(&mut server, "key".to_string(), args);
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
        let result = set_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));
        let cache = server.cache.lock().unwrap();
        assert!(cache.contains_key("key"));
    }
}
