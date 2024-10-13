use super::list_utils::ListOperation;
use crate::{log, models::value::Value, server::Server};

// TODO: handle creating a new key if key isn't found
pub fn lset_handler(server: &mut Server, key: String, args: Vec<Value>) -> Option<Value> {
    log!("lset_handler: {:?}", key);
    log!("lset_handler: {:?}", args);
    let index = match args.get(0) {
        Some(Value::Integer(i)) => *i as usize,
        _ => return Some(Value::Error("ERR index is not an integer".to_string())),
    };

    let new_value = match args.get(1) {
        Some(v) => v.clone(),
        _ => {
            return Some(Value::Error(
                "ERR wrong number of arguments for 'lset' command".to_string(),
            ))
        }
    };

    server
        .operate_on_list(&key, |list| {
            if index < list.len() {
                list[index] = new_value;
                Some(Value::SimpleString("OK".to_string()))
            } else {
                Some(Value::Error("ERR index out of range".to_string()))
            }
        })
        .or(Some(Value::Error("ERR no such key".to_string())))
}
