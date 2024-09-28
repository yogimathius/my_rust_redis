#[cfg(test)]
mod tests {

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::{del_handler, set_handler};
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn setup() -> Arc<Mutex<Server>> {
        setup_server()
    }

    #[tokio::test]
    async fn test_del_handler() {
        let server = setup();
        let args = vec![
            Value::BulkString("key1".to_string()),
            Value::BulkString("value1".to_string()),
        ];
        set_handler(server.clone(), "key1".to_string(), args).await;
        let args = vec![
            Value::BulkString("key2".to_string()),
            Value::BulkString("value2".to_string()),
        ];
        set_handler(server.clone(), "key2".to_string(), args).await;

        let args = vec![
            Value::BulkString("key1".to_string()),
            Value::BulkString("key2".to_string()),
            Value::BulkString("nonexistent_key".to_string()),
        ];
        let result = del_handler(server.clone(), "key1".to_string(), args).await;
        assert_eq!(result, Some(Value::Integer(2)));

        let args = vec![Value::BulkString("nonexistent_key".to_string())];
        let result = del_handler(server.clone(), "nonexistent_key".to_string(), args).await;
        assert_eq!(result, Some(Value::Integer(0)));
    }
}
