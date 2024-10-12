use crate::models::redis_type::RedisType;
use crate::models::value::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisItem {
    pub value: Value,
    pub created_at: i64,
    pub expiration: Option<i64>,
    pub redis_type: RedisType,
}

impl RedisItem {
    pub fn new_hash(hash: HashMap<String, Value>) -> Self {
        RedisItem {
            value: Value::Hash(hash),
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64,
            expiration: None,
            redis_type: RedisType::Hash,
        }
    }

    pub fn new_string(s: String) -> Self {
        RedisItem {
            value: Value::BulkString(s),
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64,
            expiration: None,
            redis_type: RedisType::String,
        }
    }

    pub fn new_list(list: Vec<Value>) -> Self {
        RedisItem {
            value: Value::Array(list),
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64,
            expiration: None,
            redis_type: RedisType::List,
        }
    }

    // A general constructor that can be used for any type
    pub fn new(value: Value, redis_type: RedisType) -> Self {
        RedisItem {
            value,
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64,
            expiration: None,
            redis_type,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expiration
            .map(|duration| {
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs() as i64
                    - self.created_at
                    >= duration
            })
            .unwrap_or(false)
    }
}
