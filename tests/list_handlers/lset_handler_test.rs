#[cfg(test)]
mod tests {

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::lset_handler;
    use redis_starter_rust::models::redis_type::RedisType;
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::{RedisItem, Server};
    use std::sync::Arc;
    use std::time::Instant;
    use tokio::sync::Mutex;

    async fn setup() -> Arc<Mutex<Server>> {
        setup_server()
    }

    #[tokio::test]
    async fn test_lset_handler() {
        let server = setup().await;

        // Insert a list into the cache
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

        // Test setting a value in the list
        let args = vec![
            Value::BulkString(key.clone()),
            Value::Integer(1),
            Value::BulkString("new_value".to_string()),
        ];
        let result = lset_handler(server.clone(), key.clone(), args).await;
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));

        // Verify the value was set correctly
        let server_locked = server.lock().await;
        let cache = server_locked.cache.lock().await;
        let item = cache.get(&key).unwrap();
        if let Value::Array(ref list) = item.value {
            assert_eq!(list[1], Value::BulkString("new_value".to_string()));
        } else {
            panic!("Expected list value");
        }
    }

    #[tokio::test]
    async fn test_lset_handler_index_out_of_range() {
        let server = setup().await;

        // Insert a list into the cache
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

        // Test setting a value with an out-of-range index
        let args = vec![
            Value::BulkString(key.clone()),
            Value::Integer(10),
            Value::BulkString("new_value".to_string()),
        ];
        let result = lset_handler(server.clone(), key, args).await;
        assert_eq!(
            result,
            Some(Value::Error("ERR index out of range".to_string()))
        );
    }

    #[tokio::test]
    async fn test_lset_handler_no_such_key() {
        let server = setup().await;

        // Test setting a value in a non-existent list
        let args = vec![
            Value::BulkString("non_existent_key".to_string()),
            Value::Integer(1),
            Value::BulkString("new_value".to_string()),
        ];
        let result = lset_handler(server.clone(), "non_existent_key".to_string(), args).await;
        assert_eq!(result, Some(Value::Error("ERR no such key".to_string())));
    }

    #[tokio::test]
    async fn test_lset_handler_wrong_type() {
        let server = setup().await;

        // Insert a non-list value into the cache
        let key = "key".to_string();
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

        // Test setting a value in a non-list key
        let args = vec![
            Value::BulkString(key.clone()),
            Value::Integer(1),
            Value::BulkString("new_value".to_string()),
        ];
        let result = lset_handler(server.clone(), key, args).await;
        assert_eq!(
            result,
            Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string()
            ))
        );
    }
}
