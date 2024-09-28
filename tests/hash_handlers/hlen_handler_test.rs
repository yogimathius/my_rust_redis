#[cfg(test)]
mod tests {
 

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::{hlen_handler, hset_handler};
    use redis_starter_rust::log;
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;

    fn setup() -> Server {
        return setup_server();
    }

    #[test]
    fn test_hlen_multiple_fields() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("field1".to_string()),
            Value::BulkString("value1".to_string()),
            Value::BulkString("field2".to_string()),
            Value::BulkString("value2".to_string()),
        ];
        let result = hset_handler(&mut server, "key".to_string(), args);
        log!("{:?}", result);
        let result = hlen_handler(&mut server, "key".to_string(), vec![]);
        assert_eq!(result, Some(Value::Integer(2)));
    }

    #[test]
    fn test_hlen_empty_hash() {
        let mut server = setup();
        let args = vec![];
        hset_handler(&mut server, "key".to_string(), args);
        let result = hlen_handler(&mut server, "key".to_string(), vec![]);
        assert_eq!(result, Some(Value::Integer(0)));
    }

    #[test]
    fn test_hlen_non_existent_key() {
        let mut server = setup();
        let result = hlen_handler(&mut server, "non_existent_key".to_string(), vec![]);
        assert_eq!(result, Some(Value::Integer(0)));
    }

    #[test]
    fn test_hlen_non_hash_type_key() {
        let mut server = setup();
        {
            let mut cache = server.cache.lock().unwrap();
            cache.insert(
                "key".to_string(),
                redis_starter_rust::server::RedisItem {
                    value: Value::BulkString("some string".to_string()),
                    created_at: std::time::Instant::now(),
                    expiration: None,
                    redis_type: redis_starter_rust::models::redis_type::RedisType::String,
                },
            );
        }
        let result = hlen_handler(&mut server, "key".to_string(), vec![]);
        assert_eq!(
            result,
            Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string()
            ))
        );
    }
}
