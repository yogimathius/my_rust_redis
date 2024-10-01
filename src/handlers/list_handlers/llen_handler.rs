use crate::{
    models::{redis_type::RedisType, value::Value},
    server::RedisItem,
    utilities::lock_and_get_item,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub async fn llen_handler(
    cache: Arc<Mutex<HashMap<String, RedisItem>>>,
    _key: String,
    args: Vec<Value>,
) -> Option<Value> {
    let key = match args.get(0) {
        Some(Value::BulkString(s)) => s.clone(),
        _ => {
            return Some(Value::Error(
                "ERR wrong number of arguments for 'lset' command".to_string(),
            ))
        }
    };

    match lock_and_get_item(&cache, &key, |item| {
        if let RedisType::List = item.redis_type {
            if let Value::Array(ref list) = item.value {
                Some(Value::Integer(list.len() as i64))
            } else {
                Some(Value::Error(
                    "ERR operation against a key holding the wrong kind of value".to_string(),
                ))
            }
        } else {
            Some(Value::Error(
                "ERR operation against a key holding the wrong kind of value".to_string(),
            ))
        }
    })
    .await
    {
        Ok(result) => result,
        Err(err) => Some(err),
    }
}
