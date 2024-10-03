use std::sync::Arc;

use anyhow::Error;
use redis_starter_rust::{config::Config, connection::Connection, log, server::Server};
use tokio::sync::broadcast;
struct Args {
    #[clap(short, long)]
    port: Option<u16>,
    #[clap(short, long)]
    replicaof: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let port = args.port.unwrap_or(6379);
    let server = Server::new(port, args.replicaof);

    log!("Connecting to master");
    server.run();

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
