#[cfg(test)]
mod tests {

    use std::time::Instant;

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::rpop_handler;
    use redis_starter_rust::models::redis_type::RedisType;
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::{RedisItem, Server};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    async fn setup() -> Arc<Mutex<Server>> {
        setup_server()
    }

    #[tokio::test]
    async fn test_rpop_handler_existing_list() {
        let server = setup().await;
        let key = "key".to_string();
        let initial_list = vec![
            Value::BulkString("initial".to_string()),
            Value::BulkString("second".to_string()),
        ];

        let redis_item = RedisItem {
            value: Value::Array(initial_list),
            expiration: None,
            created_at: Instant::now(),
            redis_type: RedisType::List,
        };

        server
            .lock()
            .await
            .cache
            .lock()
            .await
            .insert(key.clone(), redis_item);

        let args = vec![];

        let result = rpop_handler(server.clone(), key.clone(), args).await;
        assert_eq!(result, Some(Value::BulkString("second".to_string())));
    }
}
