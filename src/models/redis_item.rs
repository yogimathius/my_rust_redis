use serde::{Deserialize, Serialize};

use super::{redis_type::RedisType, value::Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisItem {
    pub value: Value,
    pub created_at: i64,
    pub expiration: Option<i64>,
    pub redis_type: RedisType,
}
