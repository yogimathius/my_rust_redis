#[cfg(test)]
mod tests {

    use std::time::{Duration, Instant};

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::rename_handler;
    use redis_starter_rust::models::redis_type::RedisType;
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::{RedisItem, Server};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    async fn setup() -> Arc<Mutex<Server>> {
        let server = setup_server();

        let fixed_instant = Instant::now() - Duration::from_secs(1000);
        {
            let server_locked = server.lock().await;

            server_locked.cache.lock().await.insert(
                "old_key".to_string(),
                RedisItem {
                    value: Value::BulkString("some string".to_string()),
                    created_at: fixed_instant,
                    expiration: None,
                    redis_type: RedisType::String,
                },
            );
        }

        server
    }

    fn bulk_string(value: &str) -> Value {
        Value::BulkString(value.to_string())
    }

    #[tokio::test]
    async fn test_rename_success() {
        let server = setup().await;

        let args = vec![bulk_string("old_key"), bulk_string("new_key")];
        let result = rename_handler(server.clone(), "old_key".to_string(), args).await;
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));

        let server_locked = server.lock().await;
        let cache = server_locked.cache.lock().await;
        assert!(cache.contains_key("new_key"));
        assert!(!cache.contains_key("old_key"));
        assert_eq!(
            cache.get("new_key").map(|item| &item.value),
            Some(&Value::BulkString("some string".to_string()))
        );
    }

    #[tokio::test]
    async fn test_rename_key_does_not_exist() {
        let server = setup().await;
        let args = vec![bulk_string("non_existent_key"), bulk_string("new_key")];
        let result = rename_handler(server.clone(), "non_existent_key".to_string(), args).await;
        assert_eq!(result, Some(Value::Error("ERR no such key".to_string())));
    }

    #[tokio::test]
    async fn test_rename_new_key_already_exists() {
        let server = setup().await;
        let fixed_instant = Instant::now() - Duration::from_secs(1000);
        {
            let server_locked = server.lock().await;
            let mut cache = server_locked.cache.lock().await;
            cache.insert(
                "new_key".to_string(),
                RedisItem {
                    value: Value::BulkString("existing_value".to_string()),
                    created_at: fixed_instant,
                    expiration: None,
                    redis_type: RedisType::String,
                },
            );
        }
        let args = vec![bulk_string("old_key"), bulk_string("new_key")];
        let result = rename_handler(server.clone(), "old_key".to_string(), args).await;
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));

        let server_locked = server.lock().await;
        let cache = server_locked.cache.lock().await;
        assert!(cache.contains_key("new_key"));
        assert!(!cache.contains_key("old_key"));
        assert_eq!(
            cache.get("new_key").map(|item| &item.value),
            Some(&Value::BulkString("some string".to_string()))
        );
    }
}
