#[cfg(test)]
mod tests {

    use std::time::Instant;

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::lindex_handler;
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
    async fn test_lindex_handler_get_first() {
        let server = setup().await;
        let args = vec![Value::Integer(0)];
        let result = lindex_handler(server.clone(), "key".to_string(), args).await;
        assert_eq!(result, Some(Value::BulkString("value1".to_string())));
    }

    #[tokio::test]
    async fn test_lindex_handler_get_last() {
        let server = setup().await;
        let args = vec![Value::Integer(-1)];
        let result = lindex_handler(server.clone(), "key".to_string(), args).await;
        assert_eq!(result, Some(Value::BulkString("value3".to_string())));
    }

    #[tokio::test]
    async fn test_lindex_handler_out_of_range() {
        let server = setup().await;
        let args = vec![Value::Integer(4)];
        let result = lindex_handler(server.clone(), "key".to_string(), args).await;
        assert_eq!(result, Some(Value::NullBulkString));
    }

    #[tokio::test]
    async fn test_lindex_handler_negative_one_gets_last() {
        let server = setup().await;
        let args = vec![Value::Integer(-1)];
        let result = lindex_handler(server.clone(), "key".to_string(), args).await;
        assert_eq!(result, Some(Value::BulkString("value3".to_string())));
    }

    #[tokio::test]
    async fn test_lindex_handler_negative_two_gets_second_to_last() {
        let server = setup().await;
        let args = vec![Value::Integer(-2)];
        let result = lindex_handler(server.clone(), "key".to_string(), args).await;
        assert_eq!(result, Some(Value::BulkString("value2".to_string())));
    }

    #[tokio::test]
    async fn test_lindex_handler_negative_out_of_range() {
        let server = setup().await;
        let args = vec![Value::Integer(-5)];
        let result = lindex_handler(server.clone(), "key".to_string(), args).await;
        assert_eq!(result, Some(Value::NullBulkString));
    }
}
