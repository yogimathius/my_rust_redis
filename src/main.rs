use clap::Parser;
use redis_starter_rust::server::Server;
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    port: Option<u16>,
    #[clap(short, long)]
    replicaof: Option<String>,
}
fn main() {
    let args = Args::parse();
    let port = args.port.unwrap_or(6379);

    let mut server = Server::new(port, args.replicaof);
    server.run();
    // Start the event loop
    loop {
        server.accept_connections();
        server.read_messages();
        server.process_messages();
        // Sleep for a bit to avoid monopolizing the CPU
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
