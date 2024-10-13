use crate::{handlers::list_utils::ListOperation, log, models::value::Value, server::Server};

pub fn lrem_handler(server: &mut Server, key: String, args: Vec<Value>) -> Option<Value> {
    log!("lrem_handler called with key: {} and args: {:?}", key, args);
    let count = match args.get(0) {
        Some(Value::Integer(i)) => *i,
        _ => return Some(Value::Error("ERR value is not an integer".to_string())),
    };

    let value = match args.get(1) {
        Some(Value::BulkString(v)) => v.clone(),
        _ => return Some(Value::Error("ERR value is not a bulk string".to_string())),
    };

    server
        .operate_on_list(&key, |list| {
            let mut removed = 0;
            log!("list before lrem: {:?}", list);
            list.retain(|list_item| {
                if removed == count {
                    return true;
                }
                if list_item == &Value::BulkString(value.clone()) {
                    removed += 1;
                    return false;
                }
                true
            });
            log!("list after lrem: {:?}", list);
            log!("count after lrem: {:?}", count);
            log!("removed after lrem: {:?}", removed);
            Some(Value::Integer(removed))
        })
        .or(Some(Value::Error(
            "ERR operation against a key holding the wrong kind of value".to_string(),
        )))
}
