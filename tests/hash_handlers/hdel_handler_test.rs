#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use redis_starter_rust::handlers::{hdel_handler, hset_handler};
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;
    use tokio::sync::Mutex;

    use crate::setup::setup_server;

    async fn setup() -> Arc<Mutex<Server>> {
        let server = setup_server();

        let args = vec![
            Value::BulkString("field1".to_string()),
            Value::BulkString("value1".to_string()),
            Value::BulkString("field2".to_string()),
            Value::BulkString("value2".to_string()),
        ];
        hset_handler(server.clone(), "myhash".to_string(), args).await;
        server
    }

    #[tokio::test]
    async fn test_hdel_handler() {
        let server = setup().await;
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        hset_handler(server.clone(), "key".to_string(), args).await;
        let args = vec![Value::BulkString("key".to_string())];
        let result = hdel_handler(server.clone(), "key".to_string(), args).await;
        assert_eq!(result, Some(Value::Integer(1)));
    }

    #[tokio::test]
    async fn test_hdel_handler_multiple_fields() {
        let server = setup().await;
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
            Value::BulkString("key2".to_string()),
            Value::BulkString("value2".to_string()),
        ];
        hset_handler(server.clone(), "key".to_string(), args).await;
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("key2".to_string()),
        ];
        let result = hdel_handler(server.clone(), "key".to_string(), args).await;
        assert_eq!(result, Some(Value::Integer(2)));
    }

    #[tokio::test]
    async fn test_hdel_handler_no_fields() {
        let server = setup().await;
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        hset_handler(server.clone(), "key".to_string(), args).await;
        let args = vec![];
        let result: Option<Value> = hdel_handler(server.clone(), "key".to_string(), args).await;
        assert_eq!(result, Some(Value::Integer(0)));
    }

    #[tokio::test]
    async fn test_hdel_handler_no_key() {
        let server = setup().await;
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        hset_handler(server.clone(), "key".to_string(), args).await;
        let args = vec![Value::BulkString("key2".to_string())];
        let result = hdel_handler(server.clone(), "key".to_string(), args).await;
        assert_eq!(result, Some(Value::Integer(0)));
    }
}
