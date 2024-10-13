use super::hash_utils::HashOperation;
use crate::{models::value::Value, server::Server};

pub fn hlen_handler(server: &mut Server, key: String, _: Vec<Value>) -> Option<Value> {
    server
        .operate_on_hash(&key, |hash| Some(Value::Integer(hash.len() as i64)))
        .or(Some(Value::Integer(0)))
}
