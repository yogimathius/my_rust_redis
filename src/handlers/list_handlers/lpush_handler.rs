use crate::{log, models::value::Value, server::Server, utilities::lock_and_get_item};

pub fn lpush_handler(server: &mut Server, key: String, args: Vec<Value>) -> Option<Value> {
    log!("LPUSH: Handling key '{}' with args: {:?}", key, args);

    match lock_and_get_item(&server.cache, &key, |item| match &mut item.value {
        Value::Array(list) => {
            for arg in args.iter().rev() {
                list.insert(0, arg.clone());
            }
            log!(
                "LPUSH: Updated existing list for key '{}'. New length: {}",
                key,
                list.len()
            );
            Some(Value::Integer(list.len() as i64))
        }
        _ => {
            let new_list: Vec<Value> = args.into_iter().rev().collect();
            item.value = Value::Array(new_list.clone());
            log!(
                "LPUSH: Created new list for key '{}'. Length: {}",
                key,
                new_list.len()
            );
            Some(Value::Integer(new_list.len() as i64))
        }
    }) {
        Ok(result) => {
            log!("LPUSH: Operation successful for key '{}'", key);
            result
        }
        Err(err) => {
            log!("LPUSH: Error for key '{}': {:?}", key, err);
            Some(err)
        }
    }
}
