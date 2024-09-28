use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use redis_starter_rust::server::{Role, Server};

pub fn setup_server() -> Arc<Mutex<Server>> {
    let server = Arc::new(Mutex::new(Server {
        cache: Arc::new(Mutex::new(HashMap::new())),
        role: Role::Main,
        port: 6379,
        sync: false,
        replicas: Arc::new(Mutex::new(Vec::new())),
    }));

    server
}
