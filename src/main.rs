use redis_starter_rust::{models::args::Args, server::Server};
use structopt::StructOpt;

#[tokio::main]
async fn main() {
    let args = Args::from_args();

    let mut server = Server::new(args.clone());

    server.match_replica(args.clone()).await;

    let port = args.port;

    server.listen(port).await;
}
