use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{Mutex, Notify},
    time,
};

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
            tokio::spawn(async move {
                loop {
                    let listener: Arc<tokio::net::TcpListener> = {
                        let server = server_clone.lock().await;
                        if let Some(ref listener) = server.connection_manager.listener {
                            Arc::clone(listener)
                        } else {
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
                                    server.connection_manager.connections.push(connection);
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
                let state = server.state.clone();
                server.connection_manager.read_messages(state).await;
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
                server.message_processor.process_messages().await;
                tokio::task::yield_now().await;
            }
        });
        handles.push(process_handle);

        // Periodic RDB dump task
        let server_clone = Arc::clone(&self.server);
        let rdb_dump_handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(360)); // 6 minutes
            loop {
                interval.tick().await;
                let server = server_clone.lock().await;
                if let Err(e) = server.rdb_dump().await {
                    eprintln!("Failed to perform RDB dump: {}", e);
                } else {
                    println!("RDB dump completed successfully.");
                }
            }
        });
        handles.push(rdb_dump_handle);

        handles
    }
}
