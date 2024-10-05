#[cfg(test)]
mod tests {

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::{lpop_handler, rpush_handler};
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    async fn setup() -> Arc<Mutex<Server>> {
        setup_server()
    }

    #[tokio::test]
    async fn test_lpop_handler_existing_list() {
        let server = setup().await;
        let key = "key".to_string();
        let initial_list = vec![Value::BulkString("initial".to_string())];

        rpush_handler(server.clone(), key.clone(), initial_list).await;

        let args = vec![];

        let result = lpop_handler(server.clone(), key.clone(), args).await;
        assert_eq!(result, Some(Value::BulkString("initial".to_string())));
    }
}
