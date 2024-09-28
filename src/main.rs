use clap::Parser;
use redis_starter_rust::{models::args::Args, server::Server};

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let server = Server::new(args.clone());

    server.lock().await.match_replica(args.clone()).await;

    let port = args.port;

    server.lock().await.listen(port).await;
}
