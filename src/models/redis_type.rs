use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RedisType {
    String,
    List,
    Set,
    ZSet,
    Hash,
    None,
}

impl ToString for RedisType {
    fn to_string(&self) -> String {
        match self {
            RedisType::String => "string".to_string(),
            RedisType::List => "list".to_string(),
            RedisType::Set => "set".to_string(),
            RedisType::ZSet => "zset".to_string(),
            RedisType::Hash => "hash".to_string(),
            RedisType::None => "none".to_string(),
        }
    }
}
