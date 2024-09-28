// #[cfg(test)]
// mod tests {
//     use std::collections::HashMap;
//     use std::sync::Arc;
// use tokio::sync::Mutex;

//     use redis_starter_rust::handlers::{
//         del_handler, expire_handler, get_handler, rename_handler, set_handler, type_handler,
//         unlink_handler,
//     };
//     use redis_starter_rust::models::value::Value;
//     use redis_starter_rust::server::{Role, Server};

//     fn setup() -> Server {
//         Server {
//             cache: Arc::new(Mutex::new(HashMap::new())),
//             role: Role::Main,
//             port: 6379,
//             sync: false,
//         }
//     }

//     #[test]
//     fn test_keys_handler() {
//         // let mut server = setup();
//         // let args = vec![
//         //     Value::BulkString("key1".to_string()),
//         //     Value::BulkString("value1".to_string()),
//         // ];
//         // set_handler(&mut server, "key".to_string(),  args);
//         // let args = vec![
//         //     Value::BulkString("key2".to_string()),
//         //     Value::BulkString("value2".to_string()),
//         // ];
//         // set_handler(&mut server, "key".to_string(),  args);
//         // let args = vec![Value::BulkString("*".to_string())];
//         // let result = keys_handler(&mut server, "key".to_string(),  args);
//         // // assert actual keys returned
//         // assert_eq!(
//         //     result,
//         //     Some(Value::Array(vec![
//         //         Value::SimpleString("key1".to_string()),
//         //         Value::SimpleString("key2".to_string())
//         //     ]))
//         // );

//         // This is a placeholder assertion, update it with the actual expected result
//         assert_eq!(true, true);
//     }
// }
