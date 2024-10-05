#[cfg(test)]
mod tests {

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::set_handler;
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    async fn setup() -> Arc<Mutex<Server>> {
        setup_server()
    }

    #[tokio::test]
    async fn test_set_handler() {
        let server = setup().await;
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        let result = set_handler(server.clone(), "key".to_string(), args).await;
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));
        let server_locked = server.lock().await;

        let cache = server_locked.cache.lock().await;
        assert!(cache.contains_key("key"));
    }

    #[tokio::test]
    async fn test_set_handler_with_expiration() {
        let server = setup().await;
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
            Value::BulkString("px".to_string()),
            Value::BulkString("10".to_string()),
        ];
        let result = set_handler(server.clone(), "key".to_string(), args).await;
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));
        let server_locked = server.lock().await;

        let cache = server_locked.cache.lock().await;
        assert!(cache.contains_key("key"));
    }
}
