use super::list_utils::ListOperation;
use crate::{models::value::Value, server::Server};

pub fn lpop_handler(server: &mut Server, key: String, _args: Vec<Value>) -> Option<Value> {
    server
        .operate_on_list(&key, |list| {
            if list.is_empty() {
                Some(Value::NullBulkString)
            } else {
                Some(list.remove(0))
            }
        })
        .or(Some(Value::Error(
            "ERR operation against a key holding the wrong kind of value".to_string(),
        )))
}
