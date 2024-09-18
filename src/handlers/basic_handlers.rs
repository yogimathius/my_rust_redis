use crate::{
    models::value::Value,
    server::{Role, Server},
};

pub fn ping_handler(_: &mut Server, _key: String, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("PONG".to_string()))
}

pub fn echo_handler(_: &mut Server, _key: String, args: Vec<Value>) -> Option<Value> {
    Some(args.first().unwrap().clone())
}

pub fn flushall_handler(server: &mut Server, _key: String, _: Vec<Value>) -> Option<Value> {
    let mut cache = server.cache.lock().unwrap();
    cache.clear();
    Some(Value::SimpleString("OK".to_string()))
}

pub fn info_handler(server: &Server) -> Option<Value> {
    let mut info = format!("role:{}", server.role.to_string());
    match &server.role {
        Role::Main => {
            info.push_str(&format!(
                "nmaster_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb"
            ));
            info.push_str("master_repl_offset:0");
        }
        Role::Slave { host, port } => {
            info.push_str(&format!("nmaster_host:{}nmaster_port:{}", host, port));
        }
    };
    Some(Value::BulkString(info))
}
