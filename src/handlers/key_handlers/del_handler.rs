use std::collections::HashMap;

use crate::{
    models::{redis_item::RedisItem, value::Value},
    utilities::unpack_bulk_str,
};

pub async fn del_handler(
    mut cache: HashMap<String, RedisItem>,
    _key: String,
    args: Vec<Value>,
) -> Option<Value> {
    let keys = args
        .iter()
        .map(|arg| unpack_bulk_str(&arg.clone()).unwrap())
        .collect::<Vec<String>>();

    let mut count = 0;

    for key in keys {
        if cache.remove(&key).is_some() {
            count += 1;
        }
    }

    Some(Value::Integer(count))
}
