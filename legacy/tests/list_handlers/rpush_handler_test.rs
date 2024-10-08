#[cfg(test)]
mod tests {

    use std::time::Instant;

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::rpush_handler;
    use redis_starter_rust::models::redis_type::RedisType;
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::{RedisItem, Server};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    async fn setup() -> Arc<Mutex<Server>> {
        setup_server()
    }

    #[tokio::test]
    async fn test_rpush_handler_existing_list() {
        let server = setup().await;
        let key = "key".to_string();
        let initial_list = vec![Value::BulkString("initial".to_string())];
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

        let args = vec![Value::BulkString("new_item".to_string())];
        let result = rpush_handler(server.clone(), key.clone(), args).await;
        assert_eq!(result, Some(Value::Integer(2)));

        let server_locked = server.lock().await;
        let cache = server_locked.cache.lock().await;
        if let Some(item) = cache.get(&key) {
            if let Value::Array(list) = &item.value {
                assert_eq!(list.len(), 2);
                assert_eq!(list[0], Value::BulkString("initial".to_string()));
                assert_eq!(list[1], Value::BulkString("new_item".to_string()));
            } else {
                panic!("Value is not an array");
            }
        } else {
            panic!("Key not found in cache");
        }
    }

    #[tokio::test]
    async fn test_rpush_handler_new_list() {
        let server = setup().await;
        let key = "key".to_string();
        let args = vec![Value::BulkString("new_item".to_string())];
        let result = rpush_handler(server.clone(), key.clone(), args).await;
        assert_eq!(result, Some(Value::Integer(1)));

        let server_locked = server.lock().await;
        let cache = server_locked.cache.lock().await;
        if let Some(item) = cache.get(&key) {
            if let Value::Array(list) = &item.value {
                assert_eq!(list.len(), 1);
                assert_eq!(list[0], Value::BulkString("new_item".to_string()));
            } else {
                panic!("Value is not an array");
            }
        } else {
            panic!("Key not found in cache");
        }
    }

    #[tokio::test]
    async fn test_rpush_handler_invalid_value_type() {
        let server = setup().await;
        let key = "key".to_string();
        let args = vec![Value::Integer(123)];
        let result = rpush_handler(server.clone(), key.clone(), args).await;
        assert_eq!(
            result,
            Some(Value::Error("ERR value is not a bulk string".to_string()))
        );
    }

    #[tokio::test]
    async fn test_rpush_handler_non_list_value() {
        let server = setup().await;
        let key = "key".to_string();
        let redis_item = RedisItem {
            value: Value::Integer(123),
            expiration: None,
            created_at: Instant::now(),
            redis_type: RedisType::String,
        };

        server
            .lock()
            .await
            .cache
            .lock()
            .await
            .insert(key.clone(), redis_item);

        let args = vec![Value::BulkString("new_item".to_string())];
        let result = rpush_handler(server.clone(), key.clone(), args).await;
        assert_eq!(
            result,
            Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string()
            ))
        );
    }
}