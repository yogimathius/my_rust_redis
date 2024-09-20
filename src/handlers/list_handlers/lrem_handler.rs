use crate::{models::value::Value, server::Server, utilities::lock_and_get_item};

pub fn lrem_handler(server: &mut Server, key: String, args: Vec<Value>) -> Option<Value> {
    println!("lrem_handler called with key: {} and args: {:?}", key, args);
    let count = match args.get(0) {
        Some(Value::Integer(i)) => *i,
        _ => return Some(Value::Error("ERR value is not an integer".to_string())),
    };

    let value = match args.get(1) {
        Some(Value::BulkString(v)) => v.clone(),
        _ => return Some(Value::Error("ERR value is not a bulk string".to_string())),
    };

    match lock_and_get_item(&server.cache, &key, |item| {
        if let Value::Array(ref mut list) = item.value {
            let mut removed = 0;
            println!("list before lrem: {:?}", list);
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
            println!("list after lrem: {:?}", list);
            println!("count after lrem: {:?}", count);
            println!("removed after lrem: {:?}", removed);
            Some(Value::Integer(removed))
        } else {
            Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string(),
            ))
        }
    }) {
        Ok(result) => result,
        Err(err) => Some(err),
    }
}
