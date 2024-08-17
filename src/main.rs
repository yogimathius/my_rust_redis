mod command_handler;
mod model;
mod replica_client;
mod resp;
mod server;

use crate::model::Args;
use clap::Parser;
use server::Server;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut server = Server::new(args.clone());

    server.match_replica(args.clone()).await;

    let port = args.port;

    server.listen(port).await;
}
