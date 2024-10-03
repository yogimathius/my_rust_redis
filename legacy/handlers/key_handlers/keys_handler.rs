use crate::{log, models::value::Value, server::RedisItem};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub async fn keys_handler(
    _cache: Arc<Mutex<HashMap<String, RedisItem>>>,
    _key: String,
    args: Vec<Value>,
) -> Option<Value> {
    log!("keys_handler handler {:?}", args);
    // Pseudocode:
    // 1. Extract pattern from args.
    // 2. Lock the cache.
    // 3. Iterate over keys in the cache and match them against the pattern.
    // 4. Collect matching keys.
    // 5. Return matching keys as a BulkString array.
    Some(Value::Array(vec![]))
}
