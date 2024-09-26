use crate::{log, models::value::Value, server::Server, utilities::lock_and_get_item};

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

    match lock_and_get_item(&server.cache, &key, |item| {
        if let Value::Array(ref list) = item.value {
            if index < 0 {
                let index = list.len() as i64 + index;
                if index < 0 {
                    return Some(Value::NullBulkString);
                }
                return Some(list[index as usize].clone());
            }
            if index as usize >= list.len() {
                return Some(Value::NullBulkString);
            }
            return Some(list[index as usize].clone());
        }
        Some(Value::Error(
            "ERR operation against a key holding the wrong kind of value".to_string(),
        ))
    }) {
        Ok(result) => result,
        Err(err) => Some(err),
    }
}
