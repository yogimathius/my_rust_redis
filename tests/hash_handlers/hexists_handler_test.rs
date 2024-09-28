#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::{hexists_handler, hset_handler};
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;
    use tokio::sync::Mutex;

    async fn setup() -> Arc<Mutex<Server>> {
        return setup_server();
    }

    #[tokio::test]
    async fn test_hexists_existing_field() {
        let server = setup().await;
        let args = vec![
            Value::BulkString("field".to_string()),
            Value::BulkString("value".to_string()),
        ];
        hset_handler(server.clone(), "key".to_string(), args).await;
        let args = vec![Value::BulkString("field".to_string())];
        let result = hexists_handler(server, "key".to_string(), args).await;
        assert_eq!(result, Some(Value::Integer(1)));
    }

    #[tokio::test]
    async fn test_hexists_non_existent_field() {
        let server = setup().await;
        let args = vec![
            Value::BulkString("field".to_string()),
            Value::BulkString("value".to_string()),
        ];
        hset_handler(server.clone(), "key".to_string(), args).await;
        let args = vec![Value::BulkString("non_existent_field".to_string())];
        let result = hexists_handler(server, "key".to_string(), args).await;
        assert_eq!(result, Some(Value::Integer(0)));
    }

    #[tokio::test]
    async fn test_hexists_non_existent_key() {
        let server = setup().await;
        let args = vec![Value::BulkString("field".to_string())];
        let result = hexists_handler(server, "non_existent_key".to_string(), args).await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_hexists_non_hash_type_key() {
        let server = setup().await;
        let args = vec![
            Value::BulkString("field".to_string()),
            Value::BulkString("value".to_string()),
        ];
        hset_handler(server.clone(), "key".to_string(), args).await;

        // Simulate setting the key to a different type
        {
            let server_locked = server.lock().await;
            let mut cache = server_locked.cache.lock().await;
            cache.insert(
                "key".to_string(),
                redis_starter_rust::server::RedisItem {
                    value: Value::BulkString("some string".to_string()),
                    created_at: std::time::Instant::now(),
                    expiration: None,
                    redis_type: redis_starter_rust::models::redis_type::RedisType::String,
                },
            );
        }

        let args = vec![Value::BulkString("field".to_string())];
        let result = hexists_handler(server, "key".to_string(), args).await;
        assert_eq!(
            result,
            Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string()
            ))
        );
    }

    #[tokio::test]
    async fn test_hexists_invalid_arguments() {
        let server = setup().await;
        let args = vec![
            Value::BulkString("field".to_string()),
            Value::BulkString("value".to_string()),
        ];
        hset_handler(server.clone(), "key".to_string(), args).await;
        let args = vec![Value::Integer(10)];
        let result = hexists_handler(server, "key".to_string(), args).await;
        assert_eq!(
            result,
            Some(Value::Error(
                "ERR arguments must contain a value for every field".to_string()
            ))
        );
    }
}
