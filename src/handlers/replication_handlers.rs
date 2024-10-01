use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{log, models::value::Value, replication::Role, server::Server};
use uuid::Uuid;

pub async fn replconf_handler(_: Arc<Mutex<Server>>, args: Vec<Value>) -> Option<Value> {
    log!("args: {:?}", args);
    Some(Value::SimpleString("OK".to_string()))
}

pub async fn psync_handler(server: Arc<Mutex<Server>>, _: String, _: Vec<Value>) -> Option<Value> {
    let server = server.lock().await;

    match &server.replication.role {
        Role::Main => {
            log!("server synced");
            let repl_id = generate_repl_id();
            Some(Value::SimpleString(format!("FULLRESYNC {} 0", repl_id)))
        }
        Role::Slave => {
            let msg = vec![Value::BulkString(String::from("SYNC"))];
            let payload = Value::Array(msg);
            Some(payload)
        }
    }
}

fn generate_repl_id() -> String {
    Uuid::new_v4().to_string().replace("-", "")
}
