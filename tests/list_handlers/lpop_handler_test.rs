#[cfg(test)]
mod tests {
 

    use crate::setup::setup_server;
    use redis_starter_rust::handlers::{lpop_handler, rpush_handler};
    use redis_starter_rust::models::value::Value;
    use redis_starter_rust::server::Server;

    fn setup() -> Server {
        return setup_server();
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
