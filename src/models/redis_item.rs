use crate::models::{redis_type::RedisType, value::Value};

use std::time::Instant;
#[derive(Debug, PartialEq, Clone)]
pub struct RedisItem {
    pub value: Value,
    pub created_at: Instant,
    pub expiration: Option<i64>,
    pub redis_type: RedisType,
}
