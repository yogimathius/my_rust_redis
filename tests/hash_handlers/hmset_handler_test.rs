#[cfg(test)]
mod tests {

    use redis_starter_rust::handlers::{hmset_handler, hset_handler};
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;

    use crate::setup::setup_server;
    fn setup() -> Server {
        setup_server()
    }

    #[test]
    fn test_hmset_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        hset_handler(&mut server, "key".to_string(), args);
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("10".to_string()),
        ];
        let result = hmset_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));
    }
}
