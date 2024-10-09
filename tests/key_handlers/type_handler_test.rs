#[cfg(test)]
mod tests {


    use redis_starter_rust::handlers::{set_handler, type_handler};
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;

    use crate::setup::setup_server;
    fn setup() -> Server {
        setup_server()
    }

    #[test]
    fn test_type_handler() {
        let mut server = setup();
        let args = vec![
            Value::BulkString("key".to_string()),
            Value::BulkString("value".to_string()),
        ];
        set_handler(&mut server, "key".to_string(), args);
        let args = vec![Value::BulkString("key".to_string())];
        let result = type_handler(&mut server, "key".to_string(), args);
        assert_eq!(result, Some(Value::SimpleString("string".to_string())));
    }
}
