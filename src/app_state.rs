use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{Mutex, Notify},
    time::timeout,
};

use crate::server::Server;

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
        let mut handles = Vec::new();

        let server_clone = Arc::clone(&self.server);
        let notify_clone = Arc::clone(&self.notify);

        let accept_handle = tokio::spawn(async move {
            loop {
                let _ = timeout(Duration::from_secs(1), async {
                    let mut server = server_clone.lock().await;
                    server.accept_connections().await;
                })
                .await;

                notify_clone.notify_one();
                tokio::task::yield_now().await;
            }
        });
        handles.push(accept_handle);

        // Read messages worker
        let server_clone = Arc::clone(&self.server);
        let notify_clone = Arc::clone(&self.notify);
        let read_handle = tokio::spawn(async move {
            loop {
                let _ = timeout(Duration::from_secs(5), async {
                    let mut server = server_clone.lock().await;
                    server.read_messages().await;
                })
                .await;

                notify_clone.notify_one();
                tokio::task::yield_now().await;
            }
        });
        handles.push(read_handle);

        // Process messages worker
        let server_clone = Arc::clone(&self.server);
        let process_handle = tokio::spawn(async move {
            loop {
                let _ = timeout(Duration::from_secs(5), async {
                    let mut server = server_clone.lock().await;
                    server.process_messages().await;
                })
                .await;

                tokio::task::yield_now().await;
            }
        });
        handles.push(process_handle);

        handles
    }
}
