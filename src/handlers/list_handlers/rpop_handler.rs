use super::list_utils::ListOperation;
use crate::{models::value::Value, server::Server};

pub fn rpop_handler(server: &mut Server, key: String, _args: Vec<Value>) -> Option<Value> {
    server
        .operate_on_list(&key, |list| {
            if list.is_empty() {
                Some(Value::NullBulkString)
            } else {
                Some(list.remove(list.len() - 1))
            }
        })
        .or(Some(Value::Error("ERR no such key".to_string())))
}
