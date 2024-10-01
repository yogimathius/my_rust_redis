use std::sync::Arc;

use clap::Parser;
use redis_starter_rust::{
    config::Config,
    log,
    models::{args::Args, value::Value},
    server::Server,
};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = Config::parse();

    println!("{:?}", args);
    let server = Server::new(args.clone());
    let (sender, _rx): (broadcast::Sender<Value>, _) = broadcast::channel(16);
    let sender = Arc::new(sender);
    // if let Ok(stream) = server.connect_to_master(args).await {
    //     let conn = Connection::new(stream);

    //     server.handshake(conn).await?;
    // }
    server.lock().await.match_replica(args.clone()).await;

    let port = args.port;

    log!("Starting server on port {}", port);
    server.lock().await.listen(port, sender).await;
}
