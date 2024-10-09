#[cfg(test)]
mod tests {


    use redis_starter_rust::handlers::{expire_handler, set_handler};
    use redis_starter_rust::log;
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;

    use crate::setup::setup_server;
    fn setup() -> Server {
        setup_server()
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
        log!("args {:?}", args);
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
        log!("args {:?}", args);
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
        log!("args {:?}", args);
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
        log!("args {:?}", args);
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
        log!("args {:?}", args);
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
