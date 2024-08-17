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

    match args.replicaof {
        Some(vec) => {
            let mut replica = replica_client::ReplicaClient::new(vec).await.unwrap();

            replica.send_ping(&server).await.unwrap();

            while replica.handshakes < 4 {
                match replica.read_response().await {
                    Ok(response) => {
                        replica.handshakes += 1;
                        replica.handle_response(&response, &server).await.unwrap();
                    }
                    Err(e) => {
                        eprintln!("Failed to read from stream: {}", e);
                    }
                }
            }
        }
        None => {}
    }
    let port = args.port;

    server.listen(port).await;
}
