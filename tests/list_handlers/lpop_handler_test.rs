#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use redis_starter_rust::handlers::{lpop_handler, rpush_handler};
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::{Role, Server};

    fn setup() -> Server {
        Server {
            cache: Arc::new(Mutex::new(HashMap::new())),
            role: Role::Main,
            port: 6379,
            sync: false,
        }
    }

    #[test]
    fn test_lpop_handler_existing_list() {
        let mut server = setup();
        let key = "key".to_string();
        let initial_list = vec![Value::BulkString("initial".to_string())];

        rpush_handler(&mut server, key.clone(), initial_list);

        let args = vec![];

        let result = lpop_handler(&mut server, key.clone(), args);
        assert_eq!(result, Some(Value::BulkString("initial".to_string())));
    }
}