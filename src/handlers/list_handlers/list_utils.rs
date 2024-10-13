use crate::{
    models::{redis_type::RedisType, value::Value},
    server::Server,
};

macro_rules! wrong_type_error {
    () => {
        Some(Value::Error(
            "ERR operation against a key holding the wrong kind of value".to_string(),
        ))
    };
}

pub trait ListOperation {
    fn operate_on_list<F, R>(&mut self, key: &str, f: F) -> Option<Value>
    where
        F: FnOnce(&mut Vec<Value>) -> R,
        R: Into<Option<Value>>;
}

impl ListOperation for Server {
    fn operate_on_list<F, R>(&mut self, key: &str, f: F) -> Option<Value>
    where
        F: FnOnce(&mut Vec<Value>) -> R,
        R: Into<Option<Value>>,
    {
        let mut cache = self.cache.lock().unwrap();
        match cache.get_mut(key) {
            Some(item) if item.redis_type == RedisType::List => {
                if let Value::Array(ref mut list) = item.value {
                    f(list).into()
                } else {
                    wrong_type_error!()
                }
            }
            Some(_) => wrong_type_error!(),
            None => None,
        }
    }
}
