use std::collections::HashMap;

use crate::models::redis_item::RedisItem;

pub struct Serializer;

impl Serializer {
    pub fn serialize_data(
        data: &HashMap<String, RedisItem>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();

        buffer.extend_from_slice(b"REDIS0006");

        for (key, value) in data.iter() {
            buffer.push(0x01);
            buffer.extend_from_slice(&(key.len() as u32).to_be_bytes());
            buffer.extend_from_slice(key.as_bytes());
            let serialized_value = value.serialize().expect("Failed to serialize value");
            buffer.extend_from_slice(&(serialized_value.len() as u32).to_be_bytes());
            buffer.extend_from_slice(&serialized_value);
        }

        buffer.push(0xFF);

        Ok(buffer)
    }
}
