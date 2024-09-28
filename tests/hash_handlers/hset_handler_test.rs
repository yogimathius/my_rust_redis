#[cfg(test)]
mod tests {
 
    use std::time::Instant;

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::hset_handler;
    use redis_starter_rust::models::redis_type::RedisType;
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::{RedisItem, Server};

    fn setup() -> Server {
        let mut server = setup_server();

        let args = vec![
            Value::BulkString("field1".to_string()),
            Value::BulkString("value1".to_string()),
            Value::BulkString("field2".to_string()),
            Value::BulkString("value2".to_string()),
        ];
        hset_handler(&mut server, "myhash".to_string(), args);
        server
    }

    #[test]
    fn test_hset_new_hash() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("field1".to_string()),
            Value::BulkString("value1".to_string()),
            Value::BulkString("field2".to_string()),
            Value::BulkString("value2".to_string()),
        ];
        let result = hset_handler(&mut server, "myhash".to_string(), args);
        assert_eq!(result, Some(Value::Integer(2)));

        let cache = server.cache.lock().unwrap();
        if let Some(item) = cache.get("myhash") {
            if let Value::Hash(hash) = &item.value {
                assert_eq!(
                    hash.get("field1"),
                    Some(&Value::BulkString("value1".to_string()))
                );
                assert_eq!(
                    hash.get("field2"),
                    Some(&Value::BulkString("value2".to_string()))
                );
            } else {
                panic!("Expected hash value");
            }
        } else {
            panic!("Key not found");
        }
    }

    #[test]
    fn test_hset_update_existing_field() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("field1".to_string()),
            Value::BulkString("value1".to_string()),
        ];
        hset_handler(&mut server, "myhash".to_string(), args);

        let args = vec![
            Value::BulkString("field1".to_string()),
            Value::BulkString("new_value1".to_string()),
        ];
        let result = hset_handler(&mut server, "myhash".to_string(), args);
        assert_eq!(result, Some(Value::Integer(1)));

        let cache = server.cache.lock().unwrap();
        if let Some(item) = cache.get("myhash") {
            if let Value::Hash(hash) = &item.value {
                assert_eq!(
                    hash.get("field1"),
                    Some(&Value::BulkString("new_value1".to_string()))
                );
            } else {
                panic!("Expected hash value");
            }
        } else {
            panic!("Key not found");
        }
    }

    #[test]
    fn test_hset_invalid_arguments() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("field1".to_string()),
            Value::Integer(10),
            Value::BulkString("field2".to_string()),
        ];
        let result = hset_handler(&mut server, "myhash".to_string(), args);
        assert_eq!(
            result,
            Some(Value::Error(
                "ERR arguments must contain a value for every field".to_string()
            ))
        );
    }

    #[test]
    fn test_hset_wrong_type() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("field1".to_string()),
            Value::BulkString("value1".to_string()),
        ];
        hset_handler(&mut server, "myhash".to_string(), args);

        // Simulate setting the key to a different type
        {
            let mut cache = server.cache.lock().unwrap();
            cache.insert(
                "myhash".to_string(),
                RedisItem {
                    value: Value::BulkString("some string".to_string()),
                    created_at: Instant::now(),
                    expiration: None,
                    redis_type: RedisType::String,
                },
            );
        }

        let args = vec![
            Value::BulkString("field1".to_string()),
            Value::BulkString("new_value1".to_string()),
        ];
        let result = hset_handler(&mut server, "myhash".to_string(), args);
        assert_eq!(
            result,
            Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string()
            ))
        );
    }
}
