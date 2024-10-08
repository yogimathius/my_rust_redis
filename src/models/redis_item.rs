use crate::models::{redis_type::RedisType, value::Value};

use std::time::Instant;
#[derive(Debug, PartialEq, Clone)]
pub struct RedisItem {
    pub value: Value,
    pub created_at: Instant,
    pub expiration: Option<i64>,
    pub redis_type: RedisType,
}

impl RedisItem {
    pub fn new(value: Value, expiration: Option<i64>, redis_type: RedisType) -> Self {
        RedisItem {
            value,
            created_at: Instant::now(),
            expiration,
            redis_type,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = self.expiration {
            let elapsed = self.created_at.elapsed().as_secs() as i64;
            elapsed > expiration
        } else {
            false
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        let mut buffer = Vec::new();

        buffer.extend_from_slice(&self.value.serialize_to_binary()?);

        let duration_since_epoch = self.created_at.duration_since(Instant::now());
        buffer.extend_from_slice(&duration_since_epoch.as_secs().to_be_bytes());

        match self.expiration {
            Some(exp) => {
                buffer.push(0x01);
                buffer.extend_from_slice(&exp.to_be_bytes());
            }
            None => {
                buffer.push(0x00);
            }
        }

        match self.redis_type {
            RedisType::String => buffer.push(0x01),
            RedisType::List => buffer.push(0x02),
            RedisType::Set => buffer.push(0x03),
            RedisType::ZSet => buffer.push(0x04),
            RedisType::Hash => buffer.push(0x05),
            RedisType::None => buffer.push(0x00),
        }

        Ok(buffer)
    }
}
