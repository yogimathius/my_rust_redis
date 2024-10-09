#[cfg(test)]
mod tests {


    use redis_starter_rust::handlers::{get_handler, set_handler, unlink_handler};
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;

    use crate::setup::setup_server;
    fn setup() -> Server {
        setup_server()
    }

    #[test]
    fn test_unlink_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key1".to_string()),
            Value::BulkString("value1".to_string()),
        ];
        set_handler(&mut server, "key".to_string(),  args);
        let args = vec![
            Value::BulkString("key2".to_string()),
            Value::BulkString("value2".to_string()),
        ];
        set_handler(&mut server, "key".to_string(),  args);

        let args = vec![
            Value::BulkString("key1".to_string()),
            Value::BulkString("key2".to_string()),
        ];
        let result = unlink_handler(&mut server, "key".to_string(),  args);
        assert_eq!(result, Some(Value::SimpleString("OK".to_string())));

        std::thread::sleep(std::time::Duration::from_millis(100));

        let args = vec![Value::BulkString("key1".to_string())];
        let result = get_handler(&mut server, "key".to_string(),  args);
        assert_eq!(result, Some(Value::NullBulkString));

        let args = vec![Value::BulkString("key2".to_string())];
        let result = get_handler(&mut server, "key".to_string(),  args);
        assert_eq!(result, Some(Value::NullBulkString));
    }
}
