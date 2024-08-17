use crate::{model::Value, server::Server};

pub fn hget_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn hexists_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn hdel_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn hgetall_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn hkeys_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn hlen_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn hmset_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn hset_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn hvals_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}
