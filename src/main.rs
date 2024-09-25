use clap::Parser;
use redis_starter_rust::{models::args::Args, server::Server};

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut server = Server::new(args.clone());

    server.match_replica(args.clone()).await;

    let port = args.port;

    server.listen(port).await;
}
