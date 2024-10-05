use std::sync::Arc;

use tokio::sync::{Mutex, Notify};

use crate::{log, models::connection_state::ConnectionState, server::Server};

pub struct AppState {
    pub server: Arc<Mutex<Server>>,
    pub notify: Arc<Notify>,
}

impl AppState {
    pub fn new(port: u16, replicaof: Option<String>) -> Self {
        let server = Arc::new(Mutex::new(Server::new(port, replicaof)));
        let notify = Arc::new(Notify::new());

        AppState { server, notify }
    }

    pub fn start_workers(&self) -> Vec<tokio::task::JoinHandle<()>> {
        log!("Starting workers");
        let mut handles = Vec::new();

        let server_clone = Arc::clone(&self.server);
        let notify_clone = Arc::clone(&self.notify);

        let accept_handle = {
            // let server_clone = Arc::clone(&self.server);
            // let notify_clone = Arc::clone(&self.notify);

            tokio::spawn(async move {
                // Wait until the listener is initialized
                loop {
                    let listener: Arc<tokio::net::TcpListener> = {
                        let server = server_clone.lock().await;
                        if let Some(ref listener) = server.listener {
                            Arc::clone(listener)
                        } else {
                            // Listener not initialized yet, yield and try again
                            drop(server);
                            tokio::task::yield_now().await;
                            continue;
                        }
                    };

                    loop {
                        match listener.accept().await {
                            Ok((stream, _)) => {
                                {
                                    let mut server = server_clone.lock().await;
                                    let connection = ConnectionState::new(stream);
                                    server.connections.push(connection);
                                }
                                notify_clone.notify_one();
                            }
                            Err(e) => {
                                log!("Error accepting connection: {}", e);
                            }
                        }
                    }
                }
            })
        };

        handles.push(accept_handle);

        // Read messages worker
        let server_clone = Arc::clone(&self.server);
        let notify_clone = Arc::clone(&self.notify);
        let read_handle = tokio::spawn(async move {
            loop {
                let mut server = server_clone.lock().await;
                server.read_messages().await;
                notify_clone.notify_one();
                tokio::task::yield_now().await;
            }
        });
        handles.push(read_handle);

        // Process messages worker
        let server_clone = Arc::clone(&self.server);
        let process_handle = tokio::spawn(async move {
            loop {
                let mut server = server_clone.lock().await;
                server.process_messages().await;
                tokio::task::yield_now().await;
            }
        });
        handles.push(process_handle);

        handles
    }
}
