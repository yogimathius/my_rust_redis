use crate::{
    models::{redis_type::RedisType, value::Value},
    server::Server,
};
use std::collections::HashMap;

macro_rules! wrong_type_error {
    () => {
        Some(Value::Error(
            "ERR operation against a key holding the wrong kind of value".to_string(),
        ))
    };
}

pub trait HashOperation {
    fn operate_on_hash<F, R>(&mut self, key: &str, f: F) -> Option<Value>
    where
        F: FnOnce(&mut HashMap<String, Value>) -> R,
        R: Into<Option<Value>>;
}

impl HashOperation for Server {
    fn operate_on_hash<F, R>(&mut self, key: &str, f: F) -> Option<Value>
    where
        F: FnOnce(&mut HashMap<String, Value>) -> R,
        R: Into<Option<Value>>,
    {
        let mut cache = self.cache.lock().unwrap();
        match cache.get_mut(key) {
            Some(item) if item.redis_type == RedisType::Hash => {
                if let Value::Hash(ref mut hash) = item.value {
                    f(hash).into()
                } else {
                    wrong_type_error!()
                }
            }
            Some(_) => wrong_type_error!(),
            None => None,
        }
    }
}

pub fn parse_field_value_pairs(args: &[Value]) -> Result<Vec<(String, Value)>, String> {
    args.chunks(2)
        .map(|chunk| {
            if let [Value::BulkString(field), value] = chunk {
                Ok((field.clone(), value.clone()))
            } else {
                Err("ERR arguments must contain a value for every field".to_string())
            }
        })
        .collect()
}
