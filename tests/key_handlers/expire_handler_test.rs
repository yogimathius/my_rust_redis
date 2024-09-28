#[cfg(test)]
mod tests {

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::{expire_handler, set_handler};
    use redis_starter_rust::log;
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn setup() -> Arc<Mutex<Server>> {
        setup_server()
    }

    #[tokio::test]
    async fn test_expire_handler() {
        let server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        set_handler(server.clone(), "key".to_string(), args).await;
        let args = vec![Value::BulkString("key".to_string()), Value::Integer(10)];
        log!("args {:?}", args);
        let result = expire_handler(server.clone(), "key".to_string(), args.clone()).await;
        assert_eq!(result, Some(Value::Integer(1)));
    }

    #[tokio::test]
    async fn test_expire_handler_with_nx() {
        let server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        set_handler(server.clone(), "key".to_string(), args).await;
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::Integer(10),
            Value::BulkString("NX".to_string()),
        ];
        log!("args {:?}", args);
        let result = expire_handler(server.clone(), "key".to_string(), args.clone()).await;
        assert_eq!(result, Some(Value::Integer(1)));
    }

    #[tokio::test]
    async fn test_expire_handler_with_xx() {
        let server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
            Value::BulkString("PX".to_string()),
            Value::Integer(10),
        ];
        set_handler(server.clone(), "key".to_string(), args).await;
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::Integer(10),
            Value::BulkString("XX".to_string()),
        ];
        log!("args {:?}", args);
        let result = expire_handler(server.clone(), "key".to_string(), args.clone()).await;
        assert_eq!(result, Some(Value::Integer(1)));
    }

    #[tokio::test]
    async fn test_expire_handler_with_gt() {
        let server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        set_handler(server.clone(), "key".to_string(), args).await;
        let args = vec![Value::BulkString("key".to_string()), Value::Integer(5)];
        expire_handler(server.clone(), "key".to_string(), args.clone()).await;

        let args = vec![
            Value::BulkString("key".to_string()),
            Value::Integer(10),
            Value::BulkString("GT".to_string()),
        ];
        log!("args {:?}", args);
        let result = expire_handler(server.clone(), "key".to_string(), args.clone()).await;
        assert_eq!(result, Some(Value::Integer(1)));
    }

    #[tokio::test]
    async fn test_expire_handler_with_lt() {
        let server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        set_handler(server.clone(), "key".to_string(), args).await;
        let args = vec![Value::BulkString("key".to_string()), Value::Integer(15)];
        expire_handler(server.clone(), "key".to_string(), args.clone()).await;

        let args = vec![
            Value::BulkString("key".to_string()),
            Value::Integer(10),
            Value::BulkString("LT".to_string()),
        ];
        log!("args {:?}", args);
        let result = expire_handler(server.clone(), "key".to_string(), args.clone()).await;
        assert_eq!(result, Some(Value::Integer(1)));

        let args = vec![
            Value::BulkString("key".to_string()),
            Value::Integer(10),
            Value::BulkString("LT".to_string()),
        ];

        let result = expire_handler(server.clone(), "key".to_string(), args.clone()).await;
        assert_eq!(result, Some(Value::Integer(0)));
    }
}
