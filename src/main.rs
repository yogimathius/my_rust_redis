use clap::Parser;
use redis_starter_rust::{log, models::args::Args, server::Server};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("{:?}", args);
    let server = Server::new(args.clone());

    server.lock().await.match_replica(args.clone()).await;

    let port = args.port;

    log!("Starting server on port {}", port);
    server.lock().await.listen(port).await;
}
