#[cfg(test)]
mod tests {

    use std::time::Instant;

    use redis_starter_rust::handlers::lindex_handler;
    use redis_starter_rust::models::redis_type::RedisType;
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::{RedisItem, Server};

    use crate::setup::setup_server;

    fn setup() -> Server {
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
            created_at: Instant::now().elapsed().as_secs() as i64,
            expiration: None,
            redis_type: RedisType::List,
        };
        server.cache.lock().unwrap().insert(key.clone(), redis_item);

        server
    }

    #[test]
    fn test_lindex_handler_get_first() {
        let mut server = setup();
        let args = vec![Value::Integer(0)];
        let result = lindex_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::BulkString("value1".to_string())));
    }

    #[test]
    fn test_lindex_handler_get_last() {
        let mut server = setup();
        let args = vec![Value::Integer(-1)];
        let result = lindex_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::BulkString("value3".to_string())));
    }

    #[test]
    fn test_lindex_handler_out_of_range() {
        let mut server = setup();
        let args = vec![Value::Integer(4)];
        let result = lindex_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::NullBulkString));
    }

    #[test]
    fn test_lindex_handler_negative_one_gets_last() {
        let mut server = setup();
        let args = vec![Value::Integer(-1)];
        let result = lindex_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::BulkString("value3".to_string())));
    }

    #[test]
    fn test_lindex_handler_negative_two_gets_second_to_last() {
        let mut server = setup();
        let args = vec![Value::Integer(-2)];
        let result = lindex_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::BulkString("value2".to_string())));
    }

    #[test]
    fn test_lindex_handler_negative_out_of_range() {
        let mut server = setup();
        let args = vec![Value::Integer(-5)];
        let result = lindex_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::NullBulkString));
    }
}
