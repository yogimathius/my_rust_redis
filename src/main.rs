use std::sync::Arc;

use anyhow::Error;
use redis_starter_rust::{config::Config, connection::Connection, log, server::Server};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Config::parse();
    let server = Server::new(config);
    let (sender, _rx) = broadcast::channel(16);
    let sender = Arc::new(sender);

    if let Ok(stream) = server.lock().await.connect_to_master().await {
        let conn = Connection::new(stream);

        server.lock().await.handshake(conn).await?;
    }

    let listener = server.lock().await.listen().await?;
    log!("Listening on port {}", server.lock().await.config.port);
    while let Ok((stream, _)) = listener.accept().await {
        let conn = Connection::new(stream);
        let server = Arc::clone(&server);
        let sender = Arc::clone(&sender);

        tokio::spawn(async move {
            if let Err(err) = server.lock().await.handle_connection(conn, sender).await {
                eprintln!("Failed to handle connection: {err}");
            }
        });
    }

    Ok(())
}
