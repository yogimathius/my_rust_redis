use super::list_utils::ListOperation;
use crate::{models::value::Value, server::Server};

pub fn llen_handler(server: &mut Server, key: String, _: Vec<Value>) -> Option<Value> {
    server
        .operate_on_list(&key, |list| Some(Value::Integer(list.len() as i64)))
        .or(Some(Value::Error("ERR no such key".to_string())))
}
