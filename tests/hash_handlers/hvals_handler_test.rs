#[cfg(test)]
mod tests {

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::{hset_handler, hvals_handler};
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn setup() -> Arc<Mutex<Server>> {
        setup_server()
    }

    #[tokio::test]
    async fn test_hvals_multiple_fields() {
        let server = setup();
        let args = vec![
            Value::BulkString("field1".to_string()),
            Value::BulkString("value1".to_string()),
            Value::BulkString("field2".to_string()),
            Value::BulkString("value2".to_string()),
        ];
        hset_handler(server.clone(), "key".to_string(), args).await;
        let result = hvals_handler(server.clone(), "key".to_string(), vec![]).await;
        assert_eq!(
            result,
            Some(Value::Array(vec![
                Value::BulkString("value1".to_string()),
                Value::BulkString("value2".to_string())
            ]))
        );
    }

    #[tokio::test]
    async fn test_hvals_empty_hash() {
        let server = setup();
        let args = vec![];
        hset_handler(server.clone(), "key".to_string(), args).await;
        let result = hvals_handler(server.clone(), "key".to_string(), vec![]).await;
        assert_eq!(result, Some(Value::Array(vec![])));
    }

    #[tokio::test]
    async fn test_hvals_non_existent_key() {
        let server = setup();
        let result = hvals_handler(server.clone(), "non_existent_key".to_string(), vec![]).await;
        assert_eq!(result, Some(Value::Array(vec![])));
    }

    #[tokio::test]
    async fn test_hvals_non_hash_type_key() {
        let server = setup();
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
        let result = hvals_handler(server.clone(), "key".to_string(), vec![]).await;
        assert_eq!(
            result,
            Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string()
            ))
        );
    }
}
