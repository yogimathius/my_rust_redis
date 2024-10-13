use super::list_utils::ListOperation;
use crate::{
    log,
    models::{redis_item::RedisItem, value::Value},
    server::Server,
};

pub fn lpush_handler(server: &mut Server, key: String, args: Vec<Value>) -> Option<Value> {
    log!("LPUSH: Handling key '{}' with args: {:?}", key, args);

    let result = server.operate_on_list(&key, |list| {
        for arg in args.iter().rev() {
            list.insert(0, arg.clone());
        }
        log!(
            "LPUSH: Updated existing list for key '{}'. New length: {}",
            key,
            list.len()
        );
        Some(Value::Integer(list.len() as i64))
    });

    match result {
        Some(value) => {
            log!("LPUSH: Operation successful for key '{}'", key);
            Some(value)
        }
        None => {
            // Key doesn't exist, create a new list
            let new_list: Vec<Value> = args.into_iter().rev().collect();
            let len = new_list.len();
            server
                .cache
                .lock()
                .unwrap()
                .insert(key.clone(), RedisItem::new_list(new_list));
            log!("LPUSH: Created new list for key '{}'. Length: {}", key, len);
            Some(Value::Integer(len as i64))
        }
    }
}
