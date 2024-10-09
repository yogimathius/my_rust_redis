#[cfg(test)]
mod tests {
    use std::time::Instant;

    use redis_starter_rust::handlers::{hkeys_handler, hset_handler};
    use redis_starter_rust::models::redis_type::RedisType;
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;

    use crate::setup::setup_server;
    fn setup() -> Server {
        setup_server()
    }

    #[test]
    fn test_hkeys_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("field1".to_string()),
            Value::BulkString("value1".to_string()),
            Value::BulkString("field2".to_string()),
            Value::BulkString("value2".to_string()),
        ];
        hset_handler(&mut server, "key".to_string(), args);
        let result = hkeys_handler(&mut server, "key".to_string(), vec![]);
        assert_eq!(
            result,
            Some(Value::Array(vec![
                Value::BulkString("field1".to_string()),
                Value::BulkString("field2".to_string())
            ]))
        );
    }

    #[test]
    fn test_hkeys_empty_hash() {
        let mut server = setup();
        let args = vec![];
        hset_handler(&mut server, "key".to_string(), args);
        let result = hkeys_handler(&mut server, "key".to_string(), vec![]);
        assert_eq!(result, Some(Value::Array(vec![])));
    }

    #[test]
    fn test_hkeys_non_existent_key() {
        let mut server = setup();
        let result = hkeys_handler(&mut server, "non_existent_key".to_string(), vec![]);
        assert_eq!(result, Some(Value::Array(vec![])));
    }

    #[test]
    fn test_hkeys_non_hash_type_key() {
        let mut server = setup();
        {
            let mut cache = server.cache.lock().unwrap();
            cache.insert(
                "key".to_string(),
                redis_starter_rust::server::RedisItem {
                    value: Value::BulkString("some string".to_string()),
                    created_at: Instant::now().elapsed().as_secs() as i64,
                    expiration: None,
                    redis_type: RedisType::String,
                },
            );
        }
        let result = hkeys_handler(&mut server, "key".to_string(), vec![]);
        assert_eq!(
            result,
            Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string()
            ))
        );
    }

    #[test]
    fn test_hkeys_invalid_arguments() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("field".to_string()),
            Value::BulkString("value".to_string()),
        ];
        hset_handler(&mut server, "key".to_string(), args);
        let result = hkeys_handler(&mut server, "bad_key".to_string(), vec![Value::Integer(10)]);
        assert_eq!(result, Some(Value::Array(vec![])));
    }
}
