use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    log,
    models::value::Value,
    server::{Role, Server},
    utilities::unpack_bulk_str,
};
use uuid::Uuid;

pub async fn replconf_handler(
    server: Arc<Mutex<Server>>,
    _: String,
    args: Vec<Value>,
) -> Option<Value> {
    log!("args: {:?}", args);
    let replica_port = unpack_bulk_str(args[0].clone()).unwrap();
    println!("replica_port: {}", replica_port);
    // convert string to u16
    let replica_port = replica_port.parse::<u16>().unwrap_or(0);

    if replica_port != 0 {
        let mut server = server.lock().await;
        server.replica_ports.push(replica_port);
    }
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
