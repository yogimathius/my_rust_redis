use crate::{
    models::{redis_type::RedisType, value::Value},
    server::Server,
};

pub fn type_handler(server: &mut Server, key: String, _args: Vec<Value>) -> Option<Value> {
    let cache = server.cache.lock().unwrap();
    if let Some(item) = cache.get(&key) {
        return Some(Value::SimpleString(item.redis_type.to_string()));
    } else {
        return Some(Value::SimpleString(RedisType::None.to_string()));
    }
}
