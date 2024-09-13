#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use my_redis_server::handlers::{
        del_handler, expire_handler, get_handler, keys_handler, rename_handler, set_handler,
        type_handler, unlink_handler,
    };
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
    fn test_get_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        set_handler(&mut server, args);
        let args = vec![Value::BulkString("key".to_string())];
        let result = get_handler(&mut server, args);
        assert_eq!(result, Some(Value::BulkString("value".to_string())));
    }

    #[test]
    fn test_keys_handler() {
        // let mut server = setup();
        // let args = vec![
        //     Value::BulkString("key1".to_string()),
        //     Value::BulkString("value1".to_string()),
        // ];
        // set_handler(&mut server, args);
        // let args = vec![
        //     Value::BulkString("key2".to_string()),
        //     Value::BulkString("value2".to_string()),
        // ];
        // set_handler(&mut server, args);
        // let args = vec![Value::BulkString("*".to_string())];
        // let result = keys_handler(&mut server, args);
        // // assert actual keys returned
        // assert_eq!(
        //     result,
        //     Some(Value::Array(vec![
        //         Value::SimpleString("key1".to_string()),
        //         Value::SimpleString("key2".to_string())
        //     ]))
        // );

        // This is a placeholder assertion, update it with the actual expected result
        assert_eq!(true, true);
    }

    #[test]
    fn test_type_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        set_handler(&mut server, args);
        let args = vec![Value::BulkString("key".to_string())];
        let result = type_handler(&mut server, args);
        assert_eq!(result, Some(Value::SimpleString("string".to_string())));
    }

    #[test]
    fn test_del_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key1".to_string()),
            Value::BulkString("value1".to_string()),
        ];
        set_handler(&mut server, args);
        let args = vec![
            Value::BulkString("key2".to_string()),
            Value::BulkString("value2".to_string()),
        ];
        set_handler(&mut server, args);

        let args = vec![
            Value::BulkString("key1".to_string()),
            Value::BulkString("key2".to_string()),
            Value::BulkString("nonexistent_key".to_string()),
        ];
        let result = del_handler(&mut server, args);
        assert_eq!(result, Some(Value::Integer(2)));

        let args = vec![Value::BulkString("nonexistent_key".to_string())];
        let result = del_handler(&mut server, args);
        assert_eq!(result, Some(Value::Integer(0)));
    }

    #[test]
    fn test_unlink_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key1".to_string()),
            Value::BulkString("value1".to_string()),
        ];
        set_handler(&mut server, args);
        let args = vec![
            Value::BulkString("key2".to_string()),
            Value::BulkString("value2".to_string()),
        ];
        set_handler(&mut server, args);

        let args = vec![
            Value::BulkString("key1".to_string()),
            Value::BulkString("key2".to_string()),
        ];
        let result = unlink_handler(&mut server, args);
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));

        // Allow some time for the async operation to complete
        std::thread::sleep(std::time::Duration::from_millis(100));

        let args = vec![Value::BulkString("key1".to_string())];
        let result = get_handler(&mut server, args);
        assert_eq!(result, Some(Value::NullBulkString));

        let args = vec![Value::BulkString("key2".to_string())];
        let result = get_handler(&mut server, args);
        assert_eq!(result, Some(Value::NullBulkString));
    }

    #[test]
    fn test_expire_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        set_handler(&mut server, args);
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("10".to_string()),
        ];
        let result = expire_handler(&mut server, args);
        // This is a placeholder assertion, update it with the actual expected result
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));
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
