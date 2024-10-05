#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::{lrem_handler, lset_handler};
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
    async fn test_lrem_handler_remove_existing_value() {
        let server = setup().await;
        let key = "key".to_string();

        let lrem_args = vec![Value::Integer(1), Value::BulkString("value1".to_string())];
        let result = lrem_handler(server.clone(), key.clone(), lrem_args).await;
        assert_eq!(result, Some(Value::Integer(1)));
    }

    #[tokio::test]
    async fn test_lrem_handler_remove_non_existing_value() {
        let server = setup().await;
        let key = "key".to_string();
        let args = vec![
            Value::BulkString(key.clone()),
            Value::BulkString("value".to_string()),
        ];
        lset_handler(server.clone(), key.clone(), args).await;

        let lrem_args = vec![
            Value::Integer(1),
            Value::BulkString("extra_non_existing".to_string()),
        ];
        let result = lrem_handler(server.clone(), key.clone(), lrem_args).await;
        assert_eq!(result, Some(Value::Integer(0)));
    }

    #[tokio::test]
    async fn test_lrem_handler_remove_multiple_values() {
        let server = setup().await;
        let key = "key".to_string();

        let lrem_args = vec![Value::Integer(2), Value::BulkString("value2".to_string())];
        let result = lrem_handler(server.clone(), key.clone(), lrem_args).await;
        assert_eq!(result, Some(Value::Integer(2)));
    }

    #[tokio::test]
    async fn test_lrem_handler_invalid_count() {
        let server = setup().await;
        let key = "key".to_string();
        let args = vec![
            Value::BulkString(key.clone()),
            Value::BulkString("value".to_string()),
        ];
        lset_handler(server.clone(), key.clone(), args).await;

        let lrem_args = vec![
            Value::BulkString("invalid_count".to_string()),
            Value::BulkString("value".to_string()),
        ];
        let result = lrem_handler(server.clone(), key.clone(), lrem_args).await;
        assert_eq!(
            result,
            Some(Value::Error("ERR value is not an integer".to_string()))
        );
    }

    #[tokio::test]
    async fn test_lrem_handler_invalid_value_type() {
        let server = setup().await;
        let key = "key".to_string();
        let args = vec![
            Value::BulkString(key.clone()),
            Value::BulkString("value".to_string()),
        ];
        lset_handler(server.clone(), key.clone(), args).await;

        let lrem_args = vec![Value::Integer(1), Value::Integer(123)];
        let result = lrem_handler(server.clone(), key.clone(), lrem_args).await;
        assert_eq!(
            result,
            Some(Value::Error("ERR value is not a bulk string".to_string()))
        );
    }
}
