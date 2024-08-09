mod replica_client;
mod resp;
mod server;

use clap::Parser as ClapParser;

use anyhow::Result;
use server::Server;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(ClapParser, Debug, Clone)]
struct Args {
    #[arg(short, long, default_value_t = 6379)]
    port: u16,
    #[arg(short, long, value_delimiter = ' ', num_args = 1)]
    replicaof: Option<Vec<String>>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut server = Server::new(args.clone());

    match args.replicaof {
        Some(vec) => {
            let mut replica = replica_client::ReplicaClient::new(vec).await.unwrap();

            replica.send_ping(&server).await.unwrap();

            while replica.handshakes < 2 {
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

async fn send_handshake_two(stream: &mut TcpStream, server: &Server) -> Result<()> {
    let replconf = server.replconf().unwrap();
    stream.write_all(replconf.serialize().as_bytes()).await?;
    Ok(())
}
