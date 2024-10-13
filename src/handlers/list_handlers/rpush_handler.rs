use super::list_utils::ListOperation;
use crate::{
    models::{redis_item::RedisItem, value::Value},
    server::Server,
};

pub fn rpush_handler(server: &mut Server, key: String, args: Vec<Value>) -> Option<Value> {
    if args.is_empty() {
        return Some(Value::Error(
            "ERR wrong number of arguments for 'rpush' command".to_string(),
        ));
    }

    let result = server.operate_on_list(&key, |list| {
        list.extend(args.iter().cloned());
        Some(Value::Integer(list.len() as i64))
    });

    match result {
        Some(value) => Some(value),
        None => {
            let mut cache = server.cache.lock().unwrap();
            let new_list = RedisItem::new_list(args.clone());
            cache.insert(key, new_list);
            Some(Value::Integer(args.len() as i64))
        }
    }
}
