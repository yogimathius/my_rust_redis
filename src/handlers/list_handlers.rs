use crate::{model::Value, server::Server};

pub fn llen_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn lrem_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn lindex_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn lpop_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn rpop_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

// pub fn lpush_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
//     Some(Value::SimpleString("OK".to_string()))
// }

// pub fn rpush_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
//     Some(Value::SimpleString("OK".to_string()))
// }

pub fn lset_handler(server: &mut Server, args: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}
