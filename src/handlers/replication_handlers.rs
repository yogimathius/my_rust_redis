use crate::{
    model::Value,
    server::{Role, Server},
};

pub fn replconf_handler(_: &mut Server, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn psync_handler(server: &mut Server) -> Option<Value> {
    match &server.role {
        Role::Main => None,
        Role::Slave { host: _, port: _ } => {
            server.sync = true;
            let msg = vec![Value::BulkString(String::from("SYNC"))];
            let payload = Value::Array(msg);
            Some(payload)
        }
    }
}
