#[cfg(test)]
mod tests {
 
    use std::time::Instant;

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::rpop_handler;
    use redis_starter_rust::models::redis_type::RedisType;
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::{RedisItem, Server};

    fn setup() -> Server {
        return setup_server();
    }

    #[test]
    fn test_rpop_handler_existing_list() {
        let mut server = setup();
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

        server.cache.lock().unwrap().insert(key.clone(), redis_item);

        let args = vec![];

        let result = rpop_handler(&mut server, key.clone(), args);
        assert_eq!(result, Some(Value::BulkString("second".to_string())));
    }
}
