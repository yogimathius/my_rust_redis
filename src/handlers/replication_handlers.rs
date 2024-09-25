use crate::{
    models::value::Value,
    server::{Role, Server},
};
use uuid::Uuid;

pub fn replconf_handler(_: &mut Server, _: String, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub fn psync_handler(server: &mut Server) -> Option<Value> {
    println!("Syncing with master");
    println!("server.role {:?}", server.role);

    match &server.role {
        Role::Main => {
            let repl_id = generate_repl_id();
            Some(Value::SimpleString(format!("FULLRESYNC {} 0", repl_id)))
        }
        Role::Slave { host: _, port: _ } => {
            println!("Syncing with master");
            server.sync = true;
            let msg = vec![Value::BulkString(String::from("SYNC"))];
            let payload = Value::Array(msg);
            Some(payload)
        }
    }
}

fn generate_repl_id() -> String {
    Uuid::new_v4().to_string().replace("-", "")
}
