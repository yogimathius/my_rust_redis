use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use redis_starter_rust::{
    server::{Role, Server},
    utilities::ServerState,
};

pub fn setup_server() -> Server {
    let server = Server {
        cache: Arc::new(Mutex::new(HashMap::new())),
        role: Role::Main,
        port: 6379,
        sync: false,
        server_state: ServerState::Initialising,
    };

    server
}
