use crate::{
    models::{redis_type::RedisType, value::Value},
    server::Server,
    utilities::lock_and_get_item,
};

pub fn llen_handler(server: &mut Server, key: String, _: Vec<Value>) -> Option<Value> {
    match lock_and_get_item(&server.cache, &key, |item| {
        if let RedisType::List = item.redis_type {
            if let Value::Array(ref list) = item.value {
                Some(Value::Integer(list.len() as i64))
            } else {
                Some(Value::Error(
                    "ERR operation against a key holding the wrong kind of value".to_string(),
                ))
            }
        } else {
            Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string(),
            ))
        }
    }) {
        Ok(result) => result,
        Err(err) => Some(err),
    }
}
