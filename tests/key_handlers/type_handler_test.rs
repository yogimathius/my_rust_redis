#[cfg(test)]
mod tests {

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::{set_handler, type_handler};
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    async fn setup() -> Arc<Mutex<Server>> {
        setup_server()
    }

    #[tokio::test]
    async fn test_type_handler() {
        let server = setup().await;
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        set_handler(server.clone(), "key".to_string(), args).await;
        let args = vec![Value::BulkString("key".to_string())];
        let result = type_handler(server.clone(), "key".to_string(), args).await;
        assert_eq!(result, Some(Value::SimpleString("string".to_string())));
    }
}
