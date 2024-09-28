#[cfg(test)]
mod tests {

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::{get_handler, set_handler, unlink_handler};
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::time::{sleep, Duration};

    async fn setup() -> Arc<Mutex<Server>> {
        setup_server()
    }

    #[tokio::test]
    async fn test_unlink_handler() {
        let server = setup().await;
        let args = vec![
            Value::BulkString("key1".to_string()),
            Value::BulkString("value1".to_string()),
        ];
        set_handler(server.clone(), "key".to_string(), args).await;
        let args = vec![
            Value::BulkString("key2".to_string()),
            Value::BulkString("value2".to_string()),
        ];
        set_handler(server.clone(), "key".to_string(), args).await;

        let args = vec![
            Value::BulkString("key1".to_string()),
            Value::BulkString("key2".to_string()),
        ];
        let result = unlink_handler(server.clone(), "key".to_string(), args).await;
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));

        sleep(Duration::from_millis(100)).await;

        let args = vec![Value::BulkString("key1".to_string())];
        let result = get_handler(server.clone(), "key".to_string(), args).await;
        assert_eq!(result, Some(Value::NullBulkString));

        let args = vec![Value::BulkString("key2".to_string())];
        let result = get_handler(server.clone(), "key".to_string(), args).await;
        assert_eq!(result, Some(Value::NullBulkString));
    }
}
