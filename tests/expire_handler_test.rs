#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use my_redis_server::handlers::{expire_handler, set_handler};
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
    fn test_expire_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        set_handler(&mut server, "key".to_string(), args);
        let args = vec![Value::BulkString("key".to_string()), Value::Integer(10)];
        println!("args {:?}", args);
        let result = expire_handler(&mut server, "key".to_string(), args.clone());
        assert_eq!(result, Some(Value::Integer(1)));
    }

    #[test]
    fn test_expire_handler_with_nx() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        set_handler(&mut server, "key".to_string(), args);
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::Integer(10),
            Value::BulkString("NX".to_string()),
        ];
        println!("args {:?}", args);
        let result = expire_handler(&mut server, "key".to_string(), args.clone());
        assert_eq!(result, Some(Value::Integer(1)));
    }

    #[test]
    fn test_expire_handler_with_xx() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
            Value::BulkString("PX".to_string()),
            Value::Integer(10),
        ];
        set_handler(&mut server, "key".to_string(), args);
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::Integer(10),
            Value::BulkString("XX".to_string()),
        ];
        println!("args {:?}", args);
        let result = expire_handler(&mut server, "key".to_string(), args.clone());
        assert_eq!(result, Some(Value::Integer(1)));
    }

    #[test]
    fn test_expire_handler_with_gt() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        set_handler(&mut server, "key".to_string(), args);
        let args = vec![Value::BulkString("key".to_string()), Value::Integer(5)];
        expire_handler(&mut server, "key".to_string(), args.clone());

        let args = vec![
            Value::BulkString("key".to_string()),
            Value::Integer(10),
            Value::BulkString("GT".to_string()),
        ];
        println!("args {:?}", args);
        let result = expire_handler(&mut server, "key".to_string(), args.clone());
        assert_eq!(result, Some(Value::Integer(1)));
    }

    #[test]
    fn test_expire_handler_with_lt() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        set_handler(&mut server, "key".to_string(), args);
        let args = vec![Value::BulkString("key".to_string()), Value::Integer(15)];
        expire_handler(&mut server, "key".to_string(), args.clone());

        let args = vec![
            Value::BulkString("key".to_string()),
            Value::Integer(10),
            Value::BulkString("LT".to_string()),
        ];
        println!("args {:?}", args);
        let result = expire_handler(&mut server, "key".to_string(), args.clone());
        assert_eq!(result, Some(Value::Integer(1)));

        let args = vec![
            Value::BulkString("key".to_string()),
            Value::Integer(10),
            Value::BulkString("LT".to_string()),
        ];

        let result = expire_handler(&mut server, "key".to_string(), args.clone());
        assert_eq!(result, Some(Value::Integer(0)));
    }
}
