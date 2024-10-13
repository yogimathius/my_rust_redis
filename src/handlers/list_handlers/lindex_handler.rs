use super::list_utils::ListOperation;
use crate::{log, models::value::Value, server::Server};

pub fn lindex_handler(server: &mut Server, key: String, args: Vec<Value>) -> Option<Value> {
    log!(
        "lindex_handler called with key: {} and args: {:?}",
        key,
        args,
    );

    let index = match args.get(0) {
        Some(Value::Integer(i)) => *i,
        _ => return Some(Value::Error("ERR value is not an integer".to_string())),
    };

    server.operate_on_list(&key, |list| {
        let len = list.len() as i64;
        let adjusted_index = if index < 0 { len + index } else { index };

        if adjusted_index < 0 || adjusted_index >= len {
            Some(Value::NullBulkString)
        } else {
            Some(list[adjusted_index as usize].clone())
        }
    })
}
