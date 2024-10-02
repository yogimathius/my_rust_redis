use std::sync::Arc;

use anyhow::Error;
use redis_starter_rust::{config::Config, connection::Connection, log, server::Server};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Config::parse();
    log!("Config: {:?}", config);
    let server = Server::new(config.clone());
    let (sender, _rx) = broadcast::channel(16);
    let sender = Arc::new(sender);

    log!("Connecting to master");
    if let Some(replicaof) = config.replicaof.clone() {
        let conn = Connection::new(Some(replicaof), None).await;
        log!("Handshaking with master");
        let handshake_result = {
            let mut server = server.lock().await;
            server.handshake(conn).await
        };

        if let Err(e) = handshake_result {
            eprintln!("Handshake failed: {e}");
        } else {
            log!("Handshake successful");
        }
    }

    let listener = {
        let server = server.lock().await;
        log!("Listening on port {}", server.config.port);
        server.listen().await?
    };
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                log!("Accepted connection");
                log!("Sender: {:?}", sender);
                log!("Stream: {:?}", stream);
                let conn = Connection::new(None, Some(stream)).await;
                let server = Arc::clone(&server);
                let sender = Arc::clone(&sender);

                tokio::spawn(async move {
                    if let Err(err) = server.lock().await.handle_connection(conn, sender).await {
                        log!("Failed to handle connection: {err}");
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {e}");
            }
        }
    }
}
