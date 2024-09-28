use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use redis_starter_rust::server::{Role, Server};

pub fn setup_server() -> Server {
    let mut server = Server {
        cache: Arc::new(Mutex::new(HashMap::new())),
        role: Role::Main,
        port: 6379,
        sync: false,
        replicas: Arc::new(Mutex::new(Vec::new())),
    };

    server
}
