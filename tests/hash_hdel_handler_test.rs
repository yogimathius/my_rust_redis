#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use my_redis_server::handlers::{hdel_handler, hset_handler};
    use my_redis_server::models::value::Value;
    use my_redis_server::server::{Role, Server};
    fn setup() -> Server {
        let mut server = Server {
            cache: Arc::new(Mutex::new(HashMap::new())),
            role: Role::Main,
            port: 6379,
            sync: false,
        };

        let args = vec![
            Value::BulkString("field1".to_string()),
            Value::BulkString("value1".to_string()),
            Value::BulkString("field2".to_string()),
            Value::BulkString("value2".to_string()),
        ];
        hset_handler(&mut server, "myhash".to_string(), args);
        server
    }

    #[test]
    fn test_hdel_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        hset_handler(&mut server, "key".to_string(), args);
        let args = vec![Value::BulkString("key".to_string())];
        let result = hdel_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::Integer(1)));
    }

    #[test]
    fn test_hdel_handler_multiple_fields() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
            Value::BulkString("key2".to_string()),
            Value::BulkString("value2".to_string()),
        ];
        hset_handler(&mut server, "key".to_string(), args);
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("key2".to_string()),
        ];
        let result = hdel_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::Integer(2)));
    }

    #[test]
    fn test_hdel_handler_no_fields() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        hset_handler(&mut server, "key".to_string(), args);
        let args = vec![];
        let result: Option<Value> = hdel_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::Integer(0)));
    }

    #[test]
    fn test_hdel_handler_no_key() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        hset_handler(&mut server, "key".to_string(), args);
        let args = vec![Value::BulkString("key2".to_string())];
        let result = hdel_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::Integer(0)));
    }
}
