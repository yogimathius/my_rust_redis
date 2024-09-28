use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    log,
    models::value::Value,
    server::{Role, Server},
};
use uuid::Uuid;

pub fn replconf_handler(_: Arc<Mutex<Server>>, _: String, _: Vec<Value>) -> Option<Value> {
    Some(Value::SimpleString("OK".to_string()))
}

pub async fn psync_handler(server: Arc<Mutex<Server>>, _: String, _: Vec<Value>) -> Option<Value> {
    let mut server = server.lock().await;

    match &server.role {
        Role::Main => {
            log!("server synced");
            server.sync = true;
            log!("server.sync: {}", server.sync);
            let repl_id = generate_repl_id();
            Some(Value::SimpleString(format!("FULLRESYNC {} 0", repl_id)))
        }
        Role::Slave { host: _, port: _ } => {
            let msg = vec![Value::BulkString(String::from("SYNC"))];
            let payload = Value::Array(msg);
            Some(payload)
        }
    }
}

fn generate_repl_id() -> String {
    Uuid::new_v4().to_string().replace("-", "")
}
