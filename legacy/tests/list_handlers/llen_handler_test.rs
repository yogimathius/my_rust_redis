#[cfg(test)]
mod tests {

    use std::time::Instant;

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::llen_handler;
    use redis_starter_rust::models::redis_type::RedisType;
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::{RedisItem, Server};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    async fn setup() -> Arc<Mutex<Server>> {
        let server = setup_server();
        let key = "key".to_string();
        let list = vec![
            Value::BulkString("value1".to_string()),
            Value::BulkString("value2".to_string()),
            Value::BulkString("value3".to_string()),
        ];
        let redis_item = RedisItem {
            value: Value::Array(list),
            created_at: Instant::now(),
            expiration: None,
            redis_type: RedisType::List,
        };
        server
            .lock()
            .await
            .cache
            .lock()
            .await
            .insert(key.clone(), redis_item);
        server
    }

    #[tokio::test]
    async fn test_llen_handler() {
        let server = setup().await;
        let key = "key".to_string();
        let args = vec![Value::BulkString(key.clone())];
        let result = llen_handler(server.clone(), key, args).await;
        assert_eq!(result, Some(Value::Integer(3)));
    }

    #[tokio::test]
    async fn test_llen_handler_no_key() {
        let server = setup().await;
        let key = "no_key".to_string();
        let args = vec![Value::BulkString(key.clone())];
        let result = llen_handler(server.clone(), key, args).await;
        assert_eq!(result, Some(Value::Error("ERR no such key".to_string())));
    }

    #[tokio::test]
    async fn test_llen_handler_wrong_type() {
        let server = setup().await;
        let key = "wrong_type".to_string();
        let redis_item = RedisItem {
            value: Value::BulkString("value".to_string()),
            created_at: Instant::now(),
            expiration: None,
            redis_type: RedisType::String,
        };
        server
            .lock()
            .await
            .cache
            .lock()
            .await
            .insert(key.clone(), redis_item);
        let args = vec![Value::BulkString(key.clone())];
        let result = llen_handler(server.clone(), key, args).await;
        assert_eq!(
            result,
            Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string()
            ))
        );
    }
}
